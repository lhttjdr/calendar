"""《汉书》本纪（高帝至武帝太初元年四月）所有「某年某月某干支」对秦汉初复原历（hzx.py）与现代天文定朔（meeus.py）的检验。

用法（工作目录为仓库根）：python3 scripts/calendar_data/qinhan_eval/check_hanshu.py <缓存目录>
缓存目录下缺少 001..006.txt 时自动从维基文库下载《汉书》卷一至卷六 wikitext（action=raw）。
"""
import re, sys, collections
from pathlib import Path
import urllib.request, urllib.parse
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent))
CACHE = Path(sys.argv[1] if len(sys.argv) > 1 else 'hanshu_cache'); CACHE.mkdir(parents=True, exist_ok=True)
for _v in ['001','002','003','004','005','006']:
    _f = CACHE / f'{_v}.txt'
    if not _f.exists():
        _url = 'https://zh.wikisource.org/w/index.php?title=' + urllib.parse.quote(f'漢書/卷{_v}') + '&action=raw'
        _f.write_bytes(urllib.request.urlopen(_url, timeout=60).read())
import hzx, meeus, gaiyuan_date as g
GZ=[ '甲乙丙丁戊己庚辛壬癸'[i%10]+'子丑寅卯辰巳午未申酉戌亥'[i%12] for i in range(60)]
CN={'元':1,'一':1,'二':2,'三':3,'四':4,'五':5,'六':6,'七':7,'八':8,'九':9,'十':10}
def cnnum(s):
    if s in ('元',''): return 1
    if s=='十': return 10
    if s.startswith('十'): return 10+CN[s[1]]
    if '十' in s: a,b=s.split('十'); return CN[a]*10+(CN[b] if b else 0)
    return CN[s]
WUDI={'建元':-139,'元光':-133,'元朔':-127,'元狩':-121,'元鼎':-115,'元封':-109,'太初':-103}
def N_of(vol, head):
    m=re.match(r'^(建元|元光|元朔|元狩|元鼎|元封|太初|中|後)?(元|[一二三四五六七八九十]+)年$', head)
    if not m: return None
    era, n = m.group(1), cnnum(m.group(2))
    if vol=='001': return -205+n-1
    if vol=='002': return -193+n-1
    if vol=='003': return -186+n-1
    if vol=='004': return (-162 if era=='後' else -178)+n-1
    if vol=='005': return {'中':-148,'後':-142}.get(era,-155)+n-1
    if vol=='006': return WUDI[era]+n-1 if era in WUDI else None
MONTH=re.compile(r'(?:春|夏|秋|冬)?(後九|閏|正|十二|十一|十|[二三四五六七八九])月')
TOK=re.compile(MONTH.pattern+'|('+'|'.join(GZ)+')(朔|晦)?')
stats=collections.Counter(); bad=[]
YEAR_AT_START=re.compile(r'^(建元|元光|元朔|元狩|元鼎|元封|太初)?(元|[一二三四五六七八九十]+)年')
def N_from(vol, era, n):
    if vol=='001': return -205+n-1
    if vol=='002': return -193+n-1
    if vol=='003': return -186+n-1
    if vol=='004': return (-162 if era=='後' else -178)+n-1
    if vol=='005': return {'中':-148,'後':-142}.get(era,-155)+n-1
    if vol=='006': return WUDI[era]+n-1 if era in WUDI else None
for vol in ['001','002','003','004','005','006']:
    text=open(CACHE / f'{vol}.txt',encoding='utf-8').read()
    N=None; month=None; era=None
    text=re.sub(r'\{\{YL\|([^}|]*)[^}]*\}\}', r'\1', text)
    for line in text.splitlines():
        line=line.strip()
        h=re.match(r'^==\s*(.+?)\s*==$', line)
        if h:
            head=h.group(1); month=None
            if head in ('前元','中元','後元'): era={'前元':None,'中元':'中','後元':'後'}[head]; N=None
            elif head.startswith('漢高祖'): N=-205
            elif head.startswith('秦') or head=='早年' or '贊' in head: N=None
            else:
                m=YEAR_AT_START.match(head)
                if m: N=N_from(vol, m.group(1) or era, cnnum(m.group(2)))
            continue
        m=YEAR_AT_START.match(line)
        if m and (vol!='006' or m.group(1) or era):
            if vol=='006' and m.group(1): era=m.group(1)
            N=N_from(vol, era if vol!='006' else era, cnnum(m.group(2))); month=None
        if N is None or N < -205: continue
        for t in TOK.finditer(line):
            if t.group(1):
                lab=t.group(1); sp={'後九':'后9','閏':'后9','正':'1'}; month=sp[lab] if lab in sp else str(cnnum(lab)); continue
            gz, suf = t.group(2), t.group(3)
            if month is None: stats['无月份'] += 1; continue
            if N > -103 or (N == -103 and month not in ('10','11','12','1','2','3','4')): continue  # 太初改历后属主表
            months = dict(hzx.year(N)); nxt = hzx.year(N+1)[0][1]
            labs=[l for l,_ in hzx.year(N)]
            if month not in months: stats['该年无此月'] += 1; bad.append((vol,N,month,gz,suf,'该年无此月')); continue
            i=labs.index(month); start=months[month]; end= months[labs[i+1]] if i+1<len(labs) else nxt
            off=(GZ.index(gz)-(start+49)%60)%60
            ok = start+off < end and (suf is None or (suf=='朔' and off==0) or (suf=='晦' and start+off==end-1))
            stats['合' if ok else '不合'] += 1
            # 对照：现代天文定朔（取离复原历同名月朔最近的天文朔为该月，不惩罚置闰位置差异）
            am=meeus.newmoons_between(start-20, start+50)
            a0=min(am, key=lambda a: abs(a-start)); a1=[a for a in am if a>a0][0]
            aoff=(GZ.index(gz)-(a0+49)%60)%60
            aok = a0+aoff < a1 and (suf is None or (suf=='朔' and aoff==0) or (suf=='晦' and a0+aoff==a1-1))
            stats['天文合' if aok else '天文不合'] += 1
            if suf: stats[f'{suf}:复原{"合" if ok else "不合"}/天文{"合" if aok else "不合"}'] += 1
            if not ok: bad.append((vol,N,month,gz,suf, f'该月朔{g.ganzhi_of_jdn(start)}、{end-start}日', line[max(0,t.start()-12):t.end()+4]))
print(dict(stats))
for b in bad: print(b)
