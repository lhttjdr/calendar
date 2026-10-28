"""Meeus《天文算法》第 49 章真朔（精度约分钟级），返回合朔 JDE（TT）；日界取长安地方时 UT+7.2h，ΔT 用 Morrison-Stephenson 抛物线近似。"""
import math
def true_newmoon(k):
    T=k/1236.85; JDE=2451550.09766+29.530588861*k+0.00015437*T*T-0.000000150*T**3+0.00000000073*T**4
    E=1-0.002516*T-0.0000074*T*T
    M=math.radians(2.5534+29.10535670*k-0.0000014*T*T-0.00000011*T**3)
    Mp=math.radians(201.5643+385.81693528*k+0.0107582*T*T+0.00001238*T**3-0.000000058*T**4)
    F=math.radians(160.7108+390.67050284*k-0.0016118*T*T-0.00000227*T**3+0.000000011*T**4)
    Om=math.radians(124.7746-1.56375588*k+0.0020672*T*T+0.00000215*T**3)
    c=(-0.40720*math.sin(Mp)+0.17241*E*math.sin(M)+0.01608*math.sin(2*Mp)+0.01039*math.sin(2*F)
       +0.00739*E*math.sin(Mp-M)-0.00514*E*math.sin(Mp+M)+0.00208*E*E*math.sin(2*M)-0.00111*math.sin(Mp-2*F)
       -0.00057*math.sin(Mp+2*F)+0.00056*E*math.sin(2*Mp+M)-0.00042*math.sin(3*Mp)+0.00042*E*math.sin(M+2*F)
       +0.00038*E*math.sin(M-2*F)-0.00024*E*math.sin(2*Mp-M)-0.00017*math.sin(Om))
    return JDE+c
def deltaT_days(jd):
    y=(jd-2451545)/365.25+2000; u=(y-1820)/100
    return (-20+32*u*u)/86400
def newmoon_local_jdn(k, tz_hours=7.2):
    jde=true_newmoon(k); jd=jde-deltaT_days(jde)+tz_hours/24
    return math.floor(jd+0.5)
def newmoons_between(jdn0, jdn1):
    k=math.floor((jdn0-2451550.1)/29.530588861)-1
    out=[]
    while True:
        d=newmoon_local_jdn(k); k+=1
        if d>jdn1: break
        if d>=jdn0: out.append(d)
    return out
