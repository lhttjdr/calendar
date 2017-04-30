import * as fs from 'fs';
import * as path from "path";

/* workflow
 * data file---> parse ---> calculate
 */

/*
 * VSOP (Variations Séculaires des Orbites Planétaires) 87 by Pierre Bretagnon and Gerard Francou 
 * 法国巴黎天文台天体力学和历算研究所 Institut de Mecanique Celeste et de Calculdes Ephemerides, IMCCE
 * ftp://ftp.imcce.fr/pub/ephem/planets/vsop87/
 * 
 * 版本系列
 * 
 * VSOP87總共有6個版本系列 即 ABCDE和main 這些版本的區別僅在於所選取的坐標參考係不同
 * 版本ACE為直角坐標係(xyz) BD為球面坐標系(λβδ) 版本main為橢圓坐標系（以便與VSOP82兼容）因此main有六個分
 * 量(semi-major axis, mean longitude, k = e cos(p), h = e sin(p),q = sin(g) cos(G), p = sin(g) sin(G))  
 * 所有版本當中 版本A和E很容易獲得對應的FK5-J2000星表坐標 版本C和D較易轉換為地平坐標 C為直角坐標(xyz) D為球
 * 面坐標(λβδ) 通常不使用版本main
 * 
 * 算法
 * 
 * 行星的位置P和速度V都是時間的函數 其中 V(T)=dP(T)/dT
 * VSOP給出了兩种計算P(T)途徑
 * 途徑1 P(T) = T^α*(S*Sinφ+K*Cosφ) 其中 φ=∑(i=1,12)[ai*λi]  aiλi都是與八大行星以及月球有關的攝動項
 * 途徑2 P(T) = ∑(α=0,5)∑(i=0,n)[AiT^α*Cos(Bi+CiT)]
 * 途徑2在算法實現上比途徑1簡單 因此本類採用途徑2進行計算(VSOP的官方Fortran例程亦如此 但Stellarium用的途徑1)
 * 
 * 對P(T)求導 可得V(T) 故 算法的基本公式為
 * P(T) = ∑(α=0,5)∑(i=0,n)[AiT^α*Cos(Bi+CiT)]
 * V(T) = ∑(α=0,5)∑(i=0,n)[αAiT^(α-1)*Cos(Bi+CiT)-CiAiT^α*Sin(Bi+CiT)]
 * 
 * 公式中 α=alphaT n=termsCount A=amplitude B=phase C=frequency
 * 其中α=0項為周期項 α>0項為泊松項
 * T = (JD - J2000)/365250 JD為目標時刻的儒略日TT
 * 
 * 誤差截斷
 * 
 * VSOP87所能達到的精度為 （以J2000為0點） 内行星±4000年内 土星木星±2000年内 天王星海王星±6000年内 ε<1"(arc)
 * 各行星的關聯誤差依次為 p0(i) <= 0.6e-8, 2.5e-8, 2.5e-8, 10.0e-8, 35.0e-8, 70.0e-8, 8.0e-8, 42.0e-8, i = 1..8
 * 亦即 對於某行星 其相應的真實誤差範圍是 對與距離有關的變量誤差為 p0*a0 [au|au/day] 對與角度有關的變量誤差為
 * p0 [rad|rad/day] 其中 a0為目標行星軌道半長軸的au
 * 經驗上 對某個給定的關聯誤差prec∈[p0,1e-2) 令 p(T,α) = prec/10/(-log(prec)-2)/(|T^α|+α|T^(α-1)|*10^-4) 
 * 則 對於某個α階項式L 若參量A(amplitude)滿足：|A|<p(T)（對弧度項）或|A|<a0*p(T)（對距離項） 則該項式可以忽略 
 * 這就是VSOP計算精度的控制機制
 * 
 * 輸出
 * 
 * 視版本而異 原始輸出距離單位為AU 時間單位為儒略千年
 * 對單位的轉換應該在caller内完成 典型的做法是對速度/365250 轉換到/天 對球面坐標的lb值%2π乃至進一步做象限變換
 * 
 */
 /* Definition of versions. 版本说明           
    0: VSOP87 (initial solution).
       elliptic coordinates
       dynamical equinox and ecliptic J2000.
    1: VSOP87A.
       rectangular coordinates
       heliocentric positions and velocities
       dynamical equinox and ecliptic J2000.
    2: VSOP87B.
       spherical coordinates
       heliocentric positions and velocities
       dynamical equinox and ecliptic J2000.
    3: VSOP87C.
       rectangular coordinates
       heliocentric positions and velocities
       dynamical equinox and ecliptic of the date.
    4: VSOP87D.
       spherical coordinates
       heliocentric positions and velocities
       dynamical equinox and ecliptic of the date.
    5: VSOP87E.
       rectangular coordinates
       barycentric positions and velocities
       dynamical equinox and ecliptic J2000.
 */
