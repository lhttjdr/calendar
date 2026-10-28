program Ap_Mag
! Compute several apparent magnitudes for each of the planets
! Compare the results with pre-computed values to verify correct implementation
! of the program
! This source code corresponds with the article 'Computing Apparent Planetary
! Magnitude for the Astronomical Almanac'
! submitted to 'Astronomy and Computing'
! Date: 2018 February 12
! Version 2: added equations for extened phase curves (beyond geocentric limits)
! Date: 2018 March 13
! James Hilton modified some code to standardize it

implicit none

character*200         string          ! header character string from input file
integer               mag_errors(9,2)
                    ! differences between computed and pre-determined magnitudes
                    ! first index is planet number 1 - 8 and 9 is the total
                    ! second is errors > 0.01 and > 0.001
integer               io_flag         ! IO status flag

! Initialize

! Zero the magnitude error counts
mag_errors = 0

! Open input file
open (5,file='Ap_Mag_Input_V3.txt',action='read',iostat=io_flag)
if ( io_flag /= 0 ) then
  write (0,'("Error opening input file")')
  stop
endif

! Read header line
read (5,*) string
! write (*,*) string

! Open output file for computed magnitudes and test results
open (6,file='Ap_Mag_Output_V3.txt',action='write')

! Write header lines
write (0,9) 
write (6,9)
9 format ("Ap_Mag test results")
write (0,1119)
write (6,1119)
1119 format (/,"Columns:",/,&
  "DATE = calendar date",/,"TIME = UT",/,"R = solar distance",/,  &
  "Delta = Observer distance",/,"PhAng = Phase angle",/, &
  "Sub-Obs = sub-observer longitude (Mars)",/,  &
  "Sub-Sun = sub-solar longitude (Mars)",/,  &
  "H_Ecl_Long = heliocentric ecliptic longitude (Mars)",/, &
  "Rings = rings included in magnitude T or F (Saturn)",/,  &
  "Sub-Lat = planetocentric sub-latitude (Saturn)",/, &
  "Sub-Lat = planetographic sub-latitude (Uranus)",/, &
  "AP_MAG = apparent magnitude computed here",/,  &
  "PRE_AP_MAG = pre-determined apparent magnitude",/,  &
  "DEL_AP_MAG = AP_MAG minus PRE_AP_MAG")

! Compute magnitudes for each planet
call Mercury ( mag_errors )
call Venus ( mag_errors )
call Earth ( mag_errors )
call Mars ( mag_errors )
call Jupiter ( mag_errors )
call Saturn ( mag_errors )
call Uranus ( mag_errors )
call Neptune ( mag_errors )

! Summarize the differences between computed and pre-determined magnitudes
call Error_Summary (  mag_errors )

! Finish
stop
end



Subroutine Earth ( mag_errors )
! Compute apparent magnitudes of the Earth and test them against pre-determined
! values
! Uses equation #5

implicit none

! Parameter
double precision, parameter :: phase_angle_upper_limit = 170.0
! upper limit of the phase angles for which magnitudes are thought to be
! realistic

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense'ap_mag - pre_ap_mag'
integer               mag_errors(9,2)
                  ! differences between computed and pre-determined magnitudes
                  ! first index is planet number 1 - 8 and 9 is the total
                  ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    PhAng      AP_MAG   ",  &
   "PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(39:52),'(d14.12)',err=60) r
  read (string(66:81),'(d16.14)',err=60) delta
  read (string(95:102),'(d8.4)',err=60) ph_ang
  read (string(106:111),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the phase angle factor
  ph_ang_factor = -1.060e-03 * ph_ang + 2.054e-04 * ph_ang**2

  ! Add factors to determine the apparent magnitude
  ap_mag = -3.99 + distance_mag_factor + ph_ang_factor
  
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(3,1) = mag_errors(3,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(3,2) = mag_errors(3,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ph_ang & factor                 = ",2f10.3)') ph_ang,  &
       ph_ang_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if ( ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds that for realistic magnitudes")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,3f11.3)

enddo

! Display and output error count
write (0,89) (mag_errors(1,i),i=1,2)
write (6,89) (mag_errors(1,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



Subroutine Error_Summary ( mag_errors )
! Display and output error counts

implicit none

! Computed values
integer               mag_errors(9,2)
                   ! differences between computed and  pre-determined magnitudes
                   ! first index is planet number 1 - 8 and 9 is the total
                   ! second is errors > 0.01 and > 0.001

! Misc
integer               i, j     ! loop indices
integer               io_flag  ! IO status flag

! Individual planets

write (0,9)
write (6,9)
9 format (//,"Error Summary:",/,"Planet     > 0.01   > 0.001",/)

write (0,19) (mag_errors(1,i),i=1,2)
write (6,19) (mag_errors(1,i),i=1,2)
19 format ("Mercury",6x,i1,8x,i1)

write (0,29) (mag_errors(2,i),i=1,2)
write (6,29) (mag_errors(2,i),i=1,2)
29 format ("Venus  ",6x,i1,8x,i1)

write (0,49) (mag_errors(4,i),i=1,2)
write (6,49) (mag_errors(4,i),i=1,2)
49 format ("Mars   ",6x,i1,8x,i1)

write (0,59) (mag_errors(5,i),i=1,2)
write (6,59) (mag_errors(5,i),i=1,2)
59 format ("Jupiter",6x,i1,8x,i1)

write (0,69) (mag_errors(6,i),i=1,2)
write (6,69) (mag_errors(6,i),i=1,2)
69 format ("Saturn ",6x,i1,8x,i1)

write (0,79) (mag_errors(7,i),i=1,2)
write (6,79) (mag_errors(7,i),i=1,2)
79 format ("Uranus ",6x,i1,8x,i1)

write (0,89) (mag_errors(8,i),i=1,2)
write (6,89) (mag_errors(8,i),i=1,2)
89 format ("Neptune",6x,i1,8x,i1,/)

! Summation over all planets

do i = 1, 8
  
  do j = 1, 2

    mag_errors(9,j) = mag_errors(9,j) + mag_errors(i,j) 

  enddo
  
enddo

write (0,99) (mag_errors(9,i),i=1,2)
write (6,99) (mag_errors(9,i),i=1,2)
99 format ("All    ",6x,i1,8x,i1)

! Finish
return
end



Subroutine Jupiter ( mag_errors )
! Compute apparent magnitudes of Jupiter and test them against pre-determined
! values
! Equations:
! #8 - geocentric range of phase angles
! #9 - beyond geocentric range

implicit none

! Parameters
double precision, parameter :: geocentric_phase_angle_limit = 12.0
                      ! Phase angle limit for using equations #8 and #9
double precision, parameter :: phase_angle_upper_limit = 130.0
                      ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
integer               mag_errors(9,2)
                    ! differences between computed and pre-determined magnitudes
                    ! first index is planet number 1 - 8 and 9 is the total
                    ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    PhAng      AP_MAG   ",  &
   "PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(39:52),'(d14.12)',err=60) r
  read (string(66:81),'(d16.14)',err=60) delta
  read (string(95:102),'(d8.4)',err=60) ph_ang
  read (string(106:111),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the phase angle factor
  if ( ph_ang <= geocentric_phase_angle_limit ) then
  ! Use equation #8 for phase angles below the geocentric limit
    ph_ang_factor = -3.7E-04 * ph_ang + 6.16E-04 * ph_ang**2
    else
  ! Use equation #9 for phase angles above the geocentric limit
    ph_ang_factor = -2.5 * log10( 1.0 - 1.507 * (ph_ang / 180.)    - 0.363 *  &
       (ph_ang / 180.)**2 - 0.062 * (ph_ang / 180.)**3 &
       + 2.809 * (ph_ang / 180.)**4 - 1.876 * (ph_ang / 180.)**5 )
  endif

  ! Add factors to determine the apparent magnitude
  if ( ph_ang <= geocentric_phase_angle_limit ) then
  ! Use equation #6 for phase angle <= 50 degrees
    ap_mag = -9.395 + distance_mag_factor + ph_ang_factor
  else
  ! Use equation #7 for phase angle > 50 degrees
    ap_mag = -9.428 + distance_mag_factor + ph_ang_factor
  endif
  
  ! Test of the equations #8 and #9 at the geocentric phase angle limit
  if ( .false. ) then
    ph_ang = geocentric_phase_angle_limit
    ph_ang_factor = -3.7E-04 * ph_ang + 6.16E-04 * ph_ang**2
    ap_mag = -9.395 + distance_mag_factor + ph_ang_factor
    write (0,*) ap_mag
    ph_ang_factor = -2.5 * log10( 1.0 - 1.507 * (ph_ang / 180.)    - 0.363 *  &
       (ph_ang / 180.)**2 - 0.062 * (ph_ang / 180.)**3 &
       + 2.809 * (ph_ang / 180.)**4 - 1.876 * (ph_ang / 180.)**5 )
    ap_mag = -9.428 + distance_mag_factor + ph_ang_factor
    write (0,*) ap_mag
    stop
  endif
      
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(5,1) = mag_errors(5,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(5,2) = mag_errors(5,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ph_ang & factor                 = ",2f10.3)') ph_ang,  &
       ph_ang_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if (ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds upper limit of observed ",  &
       "magnitudes")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,3f11.3)

enddo

! Display and output error count
write (0,89) (mag_errors(5,i),i=1,2)
write (6,89) (mag_errors(5,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



Subroutine Mars ( mag_errors )
! Compute apparent magnitudes of Mars and test them against pre-determined
! values
! Equations:
! #6 - geocentric range of phase angles
! #7 - beyond geocentric range

implicit none

! Parameter
double precision, parameter :: Ls_offset = -85.0
! Add to convert from helioncentric ecliptic longitude to vernal equinox
! longitude (Ls)
double precision, parameter :: geocentric_phase_angle_limit = 50.0
                      ! Phase angle limit for using equations #6 and #7
double precision, parameter :: phase_angle_upper_limit = 120.0
                      ! upper limit of the phase angle for reliable magnitudes

! Input from file
character*200         string          ! string
double precision      r               ! distance from Sun (AU)
double precision      delta           ! distance from Earth (AU)
double precision      ph_ang          ! phase angle (degrees)
character*11          date            ! calendar date
character*5           time            ! UT time
double precision      sub_earth_long  ! Sub-Earth longitude in degrees
double precision      sub_sun_long    ! Sub-Sun longitude in degrees
double precision      h_ecl_long      ! heliocentric ecliptic longitude
double precision      pre_ap_mag      ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      eff_CM
       ! Average of sub-Earth and sub-Sun longitude (effective central meridian)
double precision      Ls
! vernal equinox longitude (= 0 at vernal equinox, = 90 for north hemisphere
! summer)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
double precision      mag_corr_rot
                      ! magnitude correction for rotation angle
double precision      mag_corr_orb
                      ! magnitude correction for orbital position
integer               mag_errors(9,2)
                  ! differences between computed and pre-determined magnitudes
                  ! first index is planet number 1 - 8 and 9 is the total
                  ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta   PhAng   Sub-Obs    ",  &
   "Sub-Sun  H_Ecl_Long    AP_MAG   PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(38:43),'(d6.2)',err=60) sub_earth_long
  read (string(52:57),'(d6.2)',err=60) sub_sun_long
  read (string(66:73),'(d8.4)',err=60) h_ecl_long
  read (string(85:98),'(d14.12)',err=60) r
  read (string(112:127),'(d16.14)',err=60) delta
  read (string(153:160),'(d8.4)',err=60) ph_ang
  read (string(164:169),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor

  ! Compute the phase angle factor 
  if ( ph_ang <= geocentric_phase_angle_limit ) then
  ! Use equation #6 for phase angles below the geocentric limit
    ph_ang_factor = 2.267E-02 * ph_ang - 1.302E-04 * ph_ang**2
  else
  ! Use equation #7 for phase angles above the geocentric limit
    ph_ang_factor = - 0.02573 * ph_ang + 0.0003445 * ph_ang**2
  endif
  
  ! Compute the effective central meridian longitude
  eff_CM = ( sub_earth_long + sub_sun_long ) / 2.
  if ( abs ( sub_earth_long - sub_sun_long ) > 180. ) Eff_CM = Eff_CM + 180.
  if ( Eff_CM > 360. ) Eff_CM = Eff_CM - 360.

  ! Use Stirling interpolation to determine the magnitude correction
  call Mars_Stirling ( 'R', eff_CM, mag_corr_rot )
  
  ! Convert the ecliptic longitude to Ls
  Ls = h_ecl_long + Ls_offset
  if ( Ls > 360. ) Ls = Ls - 360.
  if ( Ls <   0. ) Ls = Ls + 360.
  
  ! Use Stirling interpolation to determine the magnitude correction
  call Mars_Stirling ( 'O', Ls, mag_corr_orb )
  
  ! Add factors to determine the apparent magnitude
  if ( ph_ang <= geocentric_phase_angle_limit ) then
  ! Use equation #6 for phase angles below the geocentric limit
    ap_mag = -1.601 + distance_mag_factor + ph_ang_factor + mag_corr_rot +  &
       mag_corr_orb
  else
  ! Use equation #7 for phase angles above the geocentric limit
    ap_mag = -0.367 + distance_mag_factor + ph_ang_factor + mag_corr_rot +  &
       mag_corr_orb
  endif
  
  ! Test of the equations #6 and #7 at the geocentric phase angle limit
  if ( .false. ) then
    ph_ang = geocentric_phase_angle_limit
    ph_ang_factor = 2.267E-02 * ph_ang - 1.302E-04 * ph_ang**2
    ap_mag = -1.601 + distance_mag_factor + ph_ang_factor + mag_corr_rot +  &
       mag_corr_orb
    write (0,*) ap_mag
    ph_ang_factor = - 0.02573 * ph_ang + 0.0003445 * ph_ang**2
    ap_mag = -0.367 + distance_mag_factor + ph_ang_factor + mag_corr_rot +  &
       mag_corr_orb
    write (0,*) ap_mag
    stop
  endif
      
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(4,1) = mag_errors(4,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(4,2) = mag_errors(4,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ph_ang & factor                 = ",2f10.3)') ph_ang,  &
       ph_ang_factor
    write (*,'("rotation & factor               = ",2f10.3)') Eff_CM,  &
       mag_corr_rot
    write (*,'("orbit & factor                  = ",2f10.3)') Ls, mag_corr_orb
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

! Display and output a warning message if the phase angle is outside the limits
! of observation
  if (ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds upper limit for reliable ",  &
       "magnitudes")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, sub_earth_long, sub_sun_long,  &
     h_ecl_long, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, sub_earth_long, sub_sun_long,  &
     h_ecl_long, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,2f8.3,2f9.3,3f11.3,3f11.3)

enddo

! Display and output error count
write (0,89) (mag_errors(4,i),i=1,2)
write (6,89) (mag_errors(4,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



subroutine Mars_Stirling (Ch, angle, mag_corr )
! Compute the magnitude correction for rotation angle or orbital position based
! on data from Mallama, 2016, Icarus, 192, 404-416
! Stirling interpolation algorithm from Duncombe in Computational Techniques in
! Explanatory Supplement to the Astronomical Almanac
! This code is based on Hilton's python code
! Example is from 'https://mat.iitm.ac.in/home/sryedida/public_html/caimna/
! interpolation/cdf.html'

implicit none

character*1              Ch
! 'R' for rotation, 'O' for orbit, 'X' for example, 'Y' for files to plot
double precision         angle        ! rotational or orbital angle, degrees
double precision         mag_corr     ! the resulting magnitude correction
integer                  i, j         ! loop indices
integer                  zero_point
                  ! mid-point in the array of 5 values from which to interpolate
double precision         p(0:5)       ! differences in angles raised to powers
double precision         delta(0:4), delta_2(0:3), delta_3(0:2), delta_4
! differences in magnitude table entries
double precision         a(0:4)       ! polynomial coefficients
double precision         L1(0:39)
! magnitude corrections for rotational longitue from Table 6 of Mallama 2007
double precision         L2(0:39)
! magnitude corrections for orbit location from Table 8 of Mallama 2007
double precision         f(0:6)       ! array for the example
logical                  L1_finished  ! flag for generating L1 and L2 files for plotting
integer, parameter :: array_offset =  0
                         ! difference between Fortran and Python

data L1 /  0.024,  0.034,  0.036,  0.045,  0.038,  0.023,  0.015,  0.011, &
           0.000, -0.012, -0.018, -0.036, -0.044, -0.059, -0.060, -0.055, &
          -0.043, -0.041, -0.041, -0.036, -0.036, -0.018, -0.038, -0.011, &
           0.002,  0.004,  0.018,  0.019,  0.035,  0.050,  0.035,  0.027, &
           0.037,  0.048,  0.025,  0.022,  0.024,  0.034,  0.036,  0.045 /

data L2 / -0.030, -0.017, -0.029, -0.017, -0.014, -0.006, -0.018, -0.020, &
          -0.014, -0.030, -0.008, -0.040, -0.024, -0.037, -0.036, -0.032, &
           0.010,  0.010, -0.001,  0.044,  0.025, -0.004, -0.016, -0.008, &
           0.029, -0.054, -0.033,  0.055,  0.017,  0.052,  0.006,  0.087, &
           0.006,  0.064,  0.030,  0.019, -0.030, -0.017, -0.029, -0.017 /

data f /0.0,     &
        0.0,     &
        0.23967, &
        0.28060, &
        0.31788, &
        0.35209, &
        0.38368 /
        
! Example case        
if (ch == 'X') angle = 22.0

! Open files for plotting and initialize
if (ch == 'Y') then
  open (11,file='L1.txt',action='write')
  open (12,file='L2.txt',action='write')
    do i = 0, 36
      write (11,9) i*10,L1(i+2)
      write (12,9) i*10,L2(i+2)
      9 format (i5,f10.3)
    enddo
  open (13,file='L1_Interp.txt',action='write')
  open (14,file='L2_Interp.txt',action='write')
  L1_finished = .false.
  angle = 0.0
endif
100 continue

! Determine the starting point for the differences and p, an array of the 
! proportional distance of the longitude between the two tabulated values
! raised to the nth power.
zero_point = int(angle / 10.0) + array_offset
p(0) = 1.0                     
p(1) = angle / 10.0 - zero_point 
do i = 2, 4                      
  p(i) = p(1)**i                 
enddo

! Calcute arrays of the first through fourth differences
do i = 0, 2                           ! for i in range(4):
  if (ch == 'R' .or. ( ch == 'Y' .and. .not. L1_finished ) ) delta(i) =  &
     L1(i + 1 + zero_point)- L1(i + zero_point)
  if (ch == 'O' .or. ( ch == 'Y' .and.       L1_finished ) ) delta(i) =  &
     L2(i + 1 + zero_point)- L2(i + zero_point)
  if (ch == 'X') delta(i) = f (i + 1 + zero_point)- f (i + zero_point)
enddo
do i = 0, 2
  delta_2(i) = delta(i + 1) - delta(i)
enddo
do i = 0, 1
  delta_3(i) = delta_2(i + 1) - delta_2(i)
enddo
delta_4 = delta_3(1) - delta_3(0)

! Convert differences into polynomial coefficients.
if (ch == 'X') a(0) = f(2 + zero_point)
if (ch == 'R' .or. ( ch == 'Y' .and. .not. L1_finished ) ) a(0) =  &
   L1(2 + zero_point)
if (ch == 'O' .or. ( ch == 'Y' .and.       L1_finished ) ) a(0) =  &
   L2(2 + zero_point)
a(4) = delta_4 / 24.0
a(3) = (delta_3(0) + delta_3(1)) / 12.0
a(2) = delta_2(1) / 2.0 - a(4)
a(1) = (delta(1) + delta(2))/ 2.0 - a(3)

! Evaluate the polynomial to compute the magnitude correction
mag_corr = 0.
do i = 0, 4
  mag_corr = mag_corr + a(i) * p(i)
enddo

! Diagnostic
if ( .false. .or. ch == 'X') then
  open (15,file="Stirling-Duncombe test output.txt",action='write')
  write ( *,109) angle
  write (15 ,109) angle
  109 format (/,"angle",f10.3)
  if (ch == 'X') then
    write ( *,114) f
    write (15,114) f
    114 format (/,"f",7f10.5)
  endif
  write ( *,119) zero_point
  write (15,119) zero_point
  119 format (/,"zero_point    ",i10)
  if (ch == 'X') then
    write ( *,129) f(zero_point+3)
    write (15,129) f(zero_point+3)
  endif
  if (ch == 'R' .or. (ch == 'Y' .and. .not. L1_finished)) then
    write ( *,129) L1(zero_point+3)
    write (15,129) L1(zero_point+3)
  endif
  if (ch == 'O' .or. (ch == 'Y' .and.       L1_finished)) then
    write ( *,129) L2(zero_point+3)
    write (15,129) L2(zero_point+3)
  endif
  129 format ("function(zero_point+3)",f10.5)
  write ( *,139) p
  write (15,139) p
  139 format ("p",5f10.5)
  write ( *,149) delta
  write (15,149) delta
  149 format (/,"delta  ",4f10.5)
  write ( *,159) delta_2
  write (15,159) delta_2
  159 format ("delta_2",3f10.5)
  write ( *,169) delta_3
  write (15,169) delta_3
  169 format ("delta_3",2f10.5)
  write ( *,179) delta_4
  write (15,179) delta_4
  179 format ("delta_4",1f10.5)
  write ( *,189) a
  write (15,189) a
  189 format (/,"a ",5f10.5)
  write ( *,199) mag_corr
  write (15,199) mag_corr
  199 format (/,"Computed value:",f10.5)
  if (ch == 'X') then
    write ( *,209)
    write (15,209)
    209 format (  "Example answer:   0.32492")
  endif
  stop
endif

! Output to plot file and increment 'angle' and L1/L2 flag if needed
if (ch == 'Y') then
  if ( .not. L1_finished ) then
    write (13,19) angle, mag_corr
    19 format (2f10.3)
    if ( angle < 359.9 ) then
      angle = angle + 1.0
    else
      L1_finished = .true.
      angle = 0.0
    endif
  else
    write (14,19) angle, mag_corr
    if ( angle < 359.9 ) then
      angle = angle + 1.0
    else
      return
    endif
  endif
  goto 100
endif  

! Finish
return
end



Subroutine Mercury ( mag_errors )
! Compute apparent magnitudes of Mercury and test them against pre-determined 
! values
! Uses equation #2

implicit none

! Parameters
double precision, parameter :: phase_angle_lower_limit = 2.0
                      ! lower limit of the observed range of phase angles
double precision, parameter :: phase_angle_upper_limit = 170.0
                      ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
integer               mag_errors(9,2)
                  ! differences between computed and  pre-determined magnitudes
                  ! first index is planet number 1 - 8 and 9 is the total
                  ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    PhAng      AP_MAG   ",  &
   "PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  write (*,39) string(1:78)
  39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(39:52),'(d14.12)',err=60) r
  read (string(66:81),'(d16.14)',err=60) delta
  read (string(95:102),'(d8.4)',err=60) ph_ang
  read (string(106:111),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the phase angle factor
  ! ph_ang_factor =  + 6.617E-02 * ph_ang    - 1.867E-03 * ph_ang**2  &
  ! + 4.103E-05 * ph_ang**3 - 4.583E-07 * ph_ang**4 &
  ! + 2.643E-09 * ph_ang**5 - 7.012E-12 * ph_ang**6 + 6.592E-15 * ph_ang**7
  
  ! 6th order
  ph_ang_factor = 6.3280e-02 * ph_ang    - 1.6336e-03 * ph_ang**2  &
     + 3.3644e-05 * ph_ang**3 &
     - 3.4265e-07 * ph_ang**4 + 1.6893e-09 * ph_ang**5 - 3.0334e-12 * ph_ang**6

  ! Add factors to determine the apparent magnitude
  ap_mag = -0.613 + distance_mag_factor + ph_ang_factor
  
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(1,1) = mag_errors(1,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(1,2) = mag_errors(1,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ph_ang & factor                 = ",2f10.3)') ph_ang,  &
       ph_ang_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if (ph_ang < phase_angle_lower_limit .or. ph_ang > phase_angle_upper_limit)  &
     then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle is outside the limits of observation")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,3f11.3)

enddo

! Display and output error count
write (0,89) (mag_errors(1,i),i=1,2)
write (6,89) (mag_errors(1,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



Subroutine Neptune ( mag_errors )
! Compute apparent magnitudes of Neptune and test them against pre-determined
! values
! Equations:
! 16 - small phase angles
! 17 - large phase angles and dates after year 2000.0

implicit none

! Parameters
double precision, parameter :: geocentric_phase_angle_limit = 1.9
                      ! Phase angle limit for using equations #6 and #7
double precision, parameter :: phase_angle_upper_limit = 133.14
                      ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag           ! difference in the sense 
                                           ! 'ap_mag - pre_ap_mag'
integer               year_int, month_int, day_int
                      ! integer part of the year, month and day
character*3           month                ! 3 letter month abbreviation
double precision      year                 ! year as a real number
integer               mag_errors(9,2)
                    ! differences between computed and pre-determined magnitudes
                    ! first index is planet number 1 - 8 and 9 is the total
                    ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    Ph_Ang  AP_MAG   ",  &
   "PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 5

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(66:80),'(d15.12)',err=60) r
  read (string(94:109),'(d16.13)',err=60) delta
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(123:130),'(f8.4)',err=60) ph_ang
  read (string(134:139),'(d6.3)',err=60) pre_ap_mag

  ! Compute a real number year
  ! Integer year and day
  read (date( 1: 4),'(i4)',err=60) year_int
  read (date(10:11),'(i2)',err=60) day_int
  ! Integer month
  month_int = 0
  read (date(6:8),'(a3)',err=60) month
  if ( month == 'Jan' ) month_int =  1
  if ( month == 'Feb' ) month_int =  2
  if ( month == 'Mar' ) month_int =  3
  if ( month == 'Apr' ) month_int =  4
  if ( month == 'May' ) month_int =  5
  if ( month == 'Jun' ) month_int =  6
  if ( month == 'Jul' ) month_int =  7
  if ( month == 'Aug' ) month_int =  8
  if ( month == 'Sep' ) month_int =  9
  if ( month == 'Oct' ) month_int = 10
  if ( month == 'Nov' ) month_int = 11
  if ( month == 'Dec' ) month_int = 12
  if ( month_int == 0 ) then
    write (*,'("Month interpretation error")')
    stop
  endif
  ! Year
  year = year_int + ( month_int - 1 ) / 12. + day_int / 365.
  
  ! diagnostic
  if ( .false. ) then
    write (*,*) date, year_int, day_int, month_int, year
    stop
  endif

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Equation 16 - compute the magnitude at unit distance as a function of time
  if ( year > 2000.0 ) ap_mag = -7.00
  if ( year < 1980.0 ) ap_mag = -6.89
  if ( year >= 1980 .and. year <= 2000.) ap_mag = -6.89 - 0.0054 * ( year -  &
     1980.0 )

  ! Add factors to determine the apparent magnitude
  ap_mag = ap_mag + distance_mag_factor
  
  ! Add phase angle factor from equation 17 if needed
  if (  ph_ang > geocentric_phase_angle_limit ) then
    ! Check the year because equation 17 only pertains to t > 2000.0
    if ( year > 2000. ) then
      ap_mag = ap_mag + 7.944e-3 * ph_ang + 9.617e-5 * ph_ang**2
    else
      write (0,279)
      write (6,279)
      279 format ("Unable to compute a magnitude for these conditions")
      write (0,79) date, time, r, delta, ph_ang
      write (6,79) date, time, r, delta, ph_ang
      goto 400
    endif
  endif

  ! Compare magnitudes at geocentric phase angle limit
  if ( .false. ) then
    ap_mag = -7.00
    write (0,*) ap_mag
    ph_ang = geocentric_phase_angle_limit
    ap_mag = ap_mag + 7.944e-3 * ph_ang + 9.617e-5 * ph_ang**2
    write (0,*) ap_mag
    stop
  endif

  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(8,1) = mag_errors(8,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(8,2) = mag_errors(8,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("year                            = ", f10.3)') year
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if (ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds upper limit of observed",  &
       " magnitudes")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,3f11.3)

  400 continue

enddo

! Display and output error count
write (0,89) (mag_errors(8,i),i=1,2)
write (6,89) (mag_errors(8,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



Subroutine Saturn ( mag_errors )
! Compute apparent magnitudes of Saturn and test them against pre-determined
! values
! Equations:
! 10 - globe and ring for geocentric values of phase angle and inclination
! 11 - globe alone for geocentric phase angles
! 12 - globe alone beyond geocentric phase angles

implicit none

! Parameters
double precision, parameter :: e2 = 0.8137d0  ! eccentricity squared of Saturn ellipse
double precision, parameter :: geocentric_phase_angle_limit = 6.5
                  ! Phase angle limit for using equations #10 and #11 versus #12
double precision, parameter :: geocentric_inclination_limit = 27.0
                  ! Inclination angle limit for using equation #10
double precision, parameter :: phase_angle_upper_limit = 150.0
                  ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude
double precision      sun_sub_lat_geod, earth_sub_lat_geod
                      ! sub-sun and sub-earth geographic latitudes
logical               rings
                      ! 'T' if for globe and rings, 'F' for globe alone

! Computed values
logical               mag_ok               ! a valid magnitude was computed
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
double precision      sun_sub_lat_geoc, earth_sub_lat_geoc
                      ! sub-sun and sub-earth geocentric latitudes
double precision      sub_lat_geoc_product, sub_lat_geoc
                      ! square root of product of sub-sun and sub-earth
                      ! geocentric latitudes and effective value
double precision      sind
                      ! The sine function in degrees
integer               mag_errors(9,2)
                  ! differences between computed and pre-determined magnitudes
                  ! first index is planet number 1 - 8 and 9 is the total
                  ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME    Rings   R     Delta    PhAng  Sub-Lat   ",  &
   "AP_MAG  PRE_AP_M  DEL_AP_M")

! Loop over HORIZON records
do i = 1, 4

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:2),'(L1)',err=60) rings
  read (string(4:14),'(a11)',err=60) date
  read (string(16:20),'(a5)',err=60) time
  read (string(47:52),'(d6.2)',err=60) sun_sub_lat_geod
  read (string(61:66),'(d6.2)',err=60) earth_sub_lat_geod
  read (string(68:82),'(d15.12)',err=60) r
  read (string(96:111),'(d16.13)',err=60) delta
  read (string(137:144),'(d8.4)',err=60) ph_ang
  read (string(148:153),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the effective sub-latitude
  ! First convert geodetic to geocentric
  call Saturn_Geodetic_Geocentric ( sun_sub_lat_geod,   sun_sub_lat_geoc,  &
     e2, .true. )
  call Saturn_Geodetic_Geocentric ( earth_sub_lat_geod, earth_sub_lat_geoc,  &
     e2, .true. )
  ! Then take the square root of the product of the saturnicentric latitude of
  ! the Sun and that of Earth but set to zero when the signs are opposite
  sub_lat_geoc_product = sun_sub_lat_geoc * earth_sub_lat_geoc
  ! Finally compute the effective value
  if ( sub_lat_geoc_product >= 0. ) then
    sub_lat_geoc = sqrt ( sub_lat_geoc_product )
  else
    sub_lat_geoc = 0.
  endif
  
  ! Set flag to false for the effect of phase angle and inclination, to be
  ! tested at the end of the 'if - else if' cases
  mag_ok = .false.

  ! Compute the effect of phase angle and inclination
  if (rings .and. ph_ang <= geocentric_phase_angle_limit .and. sub_lat_geoc  &
     <= geocentric_inclination_limit ) then
    ! Use equation #10 for globe+rings and geocentric circumstances
    ap_mag = -8.914 - 1.825 * sind ( sub_lat_geoc ) + 0.026 * ph_ang - 0.378   &
       * sind ( sub_lat_geoc ) * exp ( -2.25 * ph_ang )
    mag_ok = .true.
  else if (.not. rings .and. ph_ang <= geocentric_phase_angle_limit .and.  &
     sub_lat_geoc <= geocentric_inclination_limit ) then
    ! Use equation #11 for globe-alone and geocentric circumstances
    ap_mag = -8.95 - 3.7e-4 * ph_ang + 6.16e-4 * ph_ang**2
    mag_ok = .true.
  else if (.not. rings .and. ph_ang > geocentric_phase_angle_limit ) then
    ! Use equation #12 for globe-alone beyond geocentric phase angle limit
    ap_mag = -8.94 + 2.446e-4 * ph_ang + 2.672e-4 * ph_ang**2 - 1.506e-6 *  &
       ph_ang**3 +4.767e-9 * ph_ang**4
    mag_ok = .true.
  endif
  
  ! If the value is still zero write a warning message and continue to the next
  ! record
  if ( .not. mag_ok ) then
    write (0,279)
    write (6,279)
    279 format ("Unable to compute a magnitude for these conditions")
    write (0,79) date, time, rings, r, delta, ph_ang, sub_lat_geoc
    write (6,79) date, time, rings, r, delta, ph_ang, sub_lat_geoc
    goto 400
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if (ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds upper limit of observed",  &
       " magnitudes")
  endif

  ! Test to compare equations 10, 11 and 12 where they are joined in phase angle
  ! and with inclination set to zero
  if ( .false. ) then
    ph_ang = geocentric_phase_angle_limit
    sub_lat_geoc = 0.
    ap_mag = -8.914 - 1.825 * sind ( sub_lat_geoc ) + 0.026 * ph_ang - 0.378   &
       * sind ( sub_lat_geoc ) * exp ( -2.25 * ph_ang )
    write (0,*) ap_mag
    ap_mag = -8.95 - 3.7e-4 * ph_ang + 6.16e-4 * ph_ang**2
    write (0,*) ap_mag
    ap_mag = -8.94 + 2.446e-4 * ph_ang + 2.672e-4 * ph_ang**2 - 1.506e-6 *  &
       ph_ang**3 +4.767e-9 * ph_ang**4
    write (0,*) ap_mag
    stop
  endif      
  
  ! Add the distance factors to determine the apparent magnitude
  ap_mag = ap_mag + distance_mag_factor
  
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(6,1) = mag_errors(6,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(6,2) = mag_errors(6,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("rings logical variable          = ",5x,L1)') rings
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    ! stop
  endif

  ! Display and output to file
  write (0,79) date, time, rings, r, delta, ph_ang, sub_lat_geoc, ap_mag,  &
     pre_ap_mag, del_ap_mag
  write (6,79) date, time, rings, r, delta, ph_ang, sub_lat_geoc, ap_mag,  &
   pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,5x,L1,3x,2f7.3,2f8.3,3f10.3)

400 continue

enddo

! Display and output error count
write (0,89) (mag_errors(6,i),i=1,2)
write (6,89) (mag_errors(6,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



subroutine Saturn_Geodetic_Geocentric ( geodetic, geocentric, e2, flag )
! Convert from geodetic to geocentric latitude
! Equation #18

implicit none

double precision          geodetic    ! geodetic latitude
double precision          geocentric  ! geocentric laitude
double precision          e2
                          ! eccentricity squared of planet's elliptical figure
double precision          tand, atand
                          ! tangent and arctangent in degrees
logical                   flag
                        ! T = geodetic to geocentric, F = geocentric to geodetic

! Wiki (Saturn article): The axial tilt of Saturn to its orbit is 26.73 degrees.
! Wiki (Latitude article):
! f = (a – b) / a
! For Saturn, a = 60,268, b = 54,364. So, f = 0.0980.
! e**2 = 2 f – f**2. So,  e2 = 0.186.
! Then geocentric latitude and geodetic latitude are related by
! Geocentric = atand ( ( 1 – e**2 ) * tand (geodetic ) )
!            = atand ( 0.8137 * tan (geodetic) )

! Tested as follows:
! HORIZONS: 31.75 is the largest sub-solar planetodetic latitude from 1980-01-01
! until 1987-12-01 and after that it decreases.
! The following code gives the planetocentric sub-sun latitude as 26.73 which is
! exactly equal to the inclination quoted above.

if ( flag ) then
  ! convert geodetic to geocentric
  geocentric = atand ( e2 * tand ( geodetic ) )
else
  ! convert geocentric to geodetic
  geodetic = atand ( tand (geocentric ) / e2 )
endif

! Finish
return
end



Subroutine Uranus ( mag_errors )
! Compute apparent magnitudes of Uranus and test them against pre-determined
! values
! Equations:
! 14 - small phase angles
! 15 - large phase angles

implicit none

! Parameters
double precision, parameter :: geocentric_phase_angle_limit = 3.1
                      ! Phase angle limit for using equations #8 and #9
double precision, parameter :: phase_angle_upper_limit = 154.0
                      ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude
double precision      earth_sub_lat_planetog, sun_sub_lat_planetog
                      ! planetographic earth and Sun sub-latitudes

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
double precision      sub_lat_planetog
! average of the absolute values of the planetographic latitude of the Sun and
! that of Earth
double precision      sub_lat_factor       ! magnitude contribution of sub-lat
integer               mag_errors(9,2)
                    ! differences between computed and pre-determined magnitudes
                    ! first index is planet number 1 - 8 and 9 is the total
                    ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    PhAng  Sub-Lat  AP_MAG ",  &
   " PRE_AP_M  DEL_AP_M")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(45:50),'(d6.2)',err=60) earth_sub_lat_planetog
  read (string(59:64),'(d6.2)',err=60) sun_sub_lat_planetog
  read (string(66:80),'(d15.12)',err=60) r
  read (string(94:109),'(d16.13)',err=60) delta
  read (string(123:130),'(d8.4)',err=60) ph_ang
  read (string(134:139),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the effective sub-latitude
  ! Take the average of the absolute values of the planetographic latitude of
  ! the Sun and that of Earth
  sub_lat_planetog = ( dabs ( sun_sub_lat_planetog ) + dabs (  &
     earth_sub_lat_planetog ) ) / 2.
  
  ! Compute the sub-latitude factor
  sub_lat_factor = -0.00084 * sub_lat_planetog
  
  ! Compute the magnitude depending on the phase angle
  if ( ph_ang <= geocentric_phase_angle_limit ) then
  ! Use equation #13 for phase angles below the geocentric limit
    ap_mag = -7.110 + distance_mag_factor + sub_lat_factor
  else
  ! Use equation #14 for phase angles above the geocentric limit
    ap_mag = -7.110 + distance_mag_factor + sub_lat_factor + 6.587e-3 *   &
       ph_ang + 1.045e-4 * ph_ang**2
  endif

  ! Test the difference between equations 13 and 14 at the geocentric phase
  ! angle limit
  if ( .false. ) then
    write (0,*) 6.587e-3 * geocentric_phase_angle_limit + 1.045e-4 *  &
       geocentric_phase_angle_limit**2
    stop
  endif

  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(7,1) = mag_errors(7,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(7,2) = mag_errors(7,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("sub_lat_planetog & factor       = ",2f10.3)')   &
       sub_lat_planetog, sub_lat_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the limits of observation
  if (ph_ang > phase_angle_upper_limit) then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle exceeds upper limit of observed",  &
       " magnitudes")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, sub_lat_planetog, ap_mag,  &
      pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, sub_lat_planetog, ap_mag,  &
     pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,f7.3,3f10.3)

enddo

! Display and output error count
write (0,89) (mag_errors(7,i),i=1,2)
write (6,89) (mag_errors(7,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



Subroutine Venus ( mag_errors )
! Compute apparent magnitudes of Venus and test them against pre-determined
! values
! Equations:
! #3 - small phase angles
! #4 - large phase angles

implicit none

! Parameters
double precision, parameter :: phase_angle_lower_limit = 2.0
                      ! lower limit of the observed range of phase angles
double precision, parameter :: phase_angle_upper_limit = 179.0
                      ! upper limit of the observed range of phase angles

! Input from file
character*200         string      ! string
double precision      r           ! distance from Sun (AU)
double precision      delta       ! distance from Earth (AU)
double precision      ph_ang      ! phase angle (degrees)
character*11          date        ! calendar date
character*5           time        ! UT time
double precision      pre_ap_mag  ! predetermined apparent magnitude

! Computed values
double precision      r_mag_factor         ! distance 'r' factor in magnitudes
double precision      delta_mag_factor
                      ! distance 'delta' factor in magnitudes
double precision      distance_mag_factor
                      ! distance (r and delta) factor in magnitudes
double precision      ph_ang_factor        ! phase angle factor in magnitudes
double precision      ap_mag
                      ! apparent magnitude (Mallama et al. 2017 and later)
double precision      del_ap_mag
                      ! difference in the sense 'ap_mag - pre_ap_mag'
integer               mag_errors(9,2)
                    ! differences between computed and pre-determined magnitudes
                    ! first index is planet number 1 - 8 and 9 is the total
                    ! second is errors > 0.01 and > 0.001

! Misc
integer               i        ! loop index
integer               io_flag  ! IO status flag

! Initialize

! Read spacer record and planet identifier header record
read (5,*) string
read (5,'(a200)',iostat=io_flag) string
!write (*,9) string(1:10)
write (6,9) string
9 format (/,a10)

! Read column identifier header record
read (5,'(a200)',iostat=io_flag) string
! write (*,*) string

! Display and write column headings to output file
!write (*,29)
write (6,29)
29 format ("   DATE      TIME      R     Delta    PhAng      AP_MAG   ",  &
   "PRE_AP_MAG  DEL_AP_MAG")

! Loop over HORIZON records
do i = 1, 3

  ! Read record describing test circumstances
  read (5,'(a200)',iostat=io_flag) string
  !write (*,39) string(1:78)
  !39 format (a78)
  write (6,49) string
  49 format (a200)
  
  ! Read record containing numerical input for computation
  read (5,'(a200)',iostat=io_flag) string
  ! write (*,*) string
  
  ! Parse values
  read (string(2:12),'(a11)',err=60) date
  read (string(14:18),'(a5)',err=60) time
  read (string(39:52),'(d14.12)',err=60) r
  read (string(66:81),'(d16.14)',err=60) delta
  read (string(107:114),'(d8.4)',err=60) ph_ang
  read (string(118:123),'(d6.3)',err=60) pre_ap_mag

  ! Trap for parse error
  goto 70
  60 continue
  write (*,'("Parse error")')
  stop
  70 continue

  ! Calculate the apparent magnitude

  ! Compute the 'r' distance factor
  r_mag_factor = 2.5 * log10 ( r * r )
  
  ! Compute the 'delta' distance factor
  delta_mag_factor = 2.5 * log10 (delta * delta)

  ! Compute the distance factor
  distance_mag_factor = r_mag_factor + delta_mag_factor
 
  ! Compute the phase angle factor
  if ( ph_ang < 163.7 ) then
    ! phase angle less 163.7 degrees (equation #3)
    ph_ang_factor = -1.044E-03 * ph_ang + 3.687E-04 * ph_ang**2 - 2.814E-06 *  &
       ph_ang**3 + 8.938E-09 * ph_ang**4
    ! phase angle greater than or equal to 163.7 (equation #4)
  else 
    ph_ang_factor = 236.05828 + 4.384 - 2.81914E+00 * ph_ang + 8.39034E-03 *  &
       ph_ang**2
  endif
  
  ! Add factors to determine the apparent magnitude
  ap_mag = -4.384 + distance_mag_factor + ph_ang_factor
  
  ! Test difference computation and error counting
  if ( .false. ) then
    if ( i == 1 ) ap_mag = ap_mag + 0.02
    if ( i == 3 ) ap_mag = ap_mag - 0.002
  endif

  ! Compute the magnitude difference
  del_ap_mag = ap_mag - pre_ap_mag
  
  ! Increment the error counters if needed
  if ( abs (del_ap_mag) >  0.01 ) mag_errors(2,1) = mag_errors(2,1) + 1
  if ( abs (del_ap_mag) > 0.001 ) mag_errors(2,2) = mag_errors(2,2) + 1
  
  ! Diagnostic
  if ( .false. ) then
    write (*,'("date & time                     = ",5x,a11,2x,a5)') date, time
    write (*,'("r & factor                      = ",2f10.3)') r, r_mag_factor
    write (*,'("delta & factor                  = ",2f10.3)') delta,  &
       delta_mag_factor
    write (*,'("distance factor                 = ",2f10.3)')  &
       distance_mag_factor
    write (*,'("ph_ang & factor                 = ",2f10.3)') ph_ang,  &
       ph_ang_factor
    write (*,'("ap_mag, pre_ap_mag & del_ap_mag = ",3f10.3)') ap_mag,  &
       pre_ap_mag, del_ap_mag
    stop
  endif

  ! Display and output a warning message if the phase angle is outside the
  ! limits of observation
  if (ph_ang < phase_angle_lower_limit .or. ph_ang > phase_angle_upper_limit)  &
     then
    write (0,179)
    write (6,179)
    179 format ("Warning: phase angle is outside the limits of observation")
  endif

  ! Display and output to file
  write (0,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  write (6,79) date, time, r, delta, ph_ang, ap_mag, pre_ap_mag, del_ap_mag
  79 format (a11,2x,a5,3x,2f7.3,f8.3,3f11.3)

enddo

! Display and output error count
write (0,89) (mag_errors(2,i),i=1,2)
write (6,89) (mag_errors(2,i),i=1,2)
89 format ("Magnitude errors > 0.01 = ",i1,", magnitude errors > 0.001 = ",i1)

! Finish
return
end



double precision Function tand (theta)
! Compute the tangent function of an angle, theta, given in degrees.
   implicit none

! Parameters
double precision, parameter :: deg_to_rad = 0.017453292519943
                                     ! conversion factor from degrees to radians

   double precision theta  ! The angle in degrees

   tand = tan (theta * deg_to_rad)

! Finish
end



double precision Function sind (theta)
! Compute the sine function of an angle, theta, given in degrees.
   implicit none

! Parameters
double precision, parameter :: deg_to_rad = 0.017453292519943
                                     ! conversion factor from degrees to radians

   double precision theta  ! The angle in degrees

   sind = sin (theta * deg_to_rad)

! Finish
end



double precision Function atand (r)
! Compute the inverse tangent function of an angle, theta, given in degrees.
   implicit none

! Parameters
double precision, parameter :: rad_to_deg = 57.295779513082321
                                     ! conversion factor from radians to degrees

   double precision r  ! The tangent of the angle

   atand = rad_to_deg * atan (r)

! Finish
end

