#!/bin/bash
# Recalibrate `compare`'s solo-agreement allowance from a deliberate triple.
# The 5% in compare.rs is set from three ramps that happened to run back to
# back; this runs the same ramp three times on purpose so the spread of the
# solo rung is measured rather than inherited.
cd /c/Users/LilMG/Desktop/coggy
SB="C:/Users/LilMG/Desktop/coggy/target/release/sessionbench.exe"
W='C:\Users\LilMG\Desktop\coggy\target\release\cpu-spin.exe'
for i in 1 2 3; do
  echo "########## repeat $i/3 ##########"
  "$SB" ramp --label calib-$i --hold 60 --max-sessions 60 --resolution 2 --out bench-out \
    -- "$W" --units 1000000
done
echo "########## done — now compare each pair ##########"
