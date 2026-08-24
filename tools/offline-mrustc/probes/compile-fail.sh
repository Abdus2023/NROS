#!/bin/bash
# Compile-fail probes (Pass 27) — each of these MUST be rejected by the compiler,
# mirroring the cargo trybuild suite in crates/nros-core/tests (F-19 notes the .stderr
# blessing gap is a trybuild-workflow issue under real cargo; the underlying rejection
# property is probed here, toolchain-independently).
set -u
T=${TOOLCHAIN_ROOT:-/home/user/toolchain}
NROS=${NROS_ROOT:-/home/user/NROS}
export RUSTC_VERSION=1.90.0 MRUSTC_TARGET_VER=1.90 OUTDIR_SUF=-1.90.0
MRUSTC=$T/mrustc-master/bin/mrustc
STD=$T/mrustc-master/output-1.90.0
NOUT=$T/nros-out
mkdir -p $NOUT/cf
fail=0
for f in $NROS/crates/nros-core/tests/compile_fail/*.rs; do
  name=$(basename "$f" .rs)
  if $MRUSTC "$f" -o $NOUT/cf/$name --crate-name cf_$name --crate-type bin \
       --edition 2021 -O -L "$STD" -L "$NOUT" \
       --extern nros_types=$NOUT/libnros_types.rlib \
       --extern nros_core=$NOUT/libnros_core.rlib > $NOUT/cf/$name.dbg.txt 2>&1; then
    echo "!!! UNEXPECTED ACCEPT: $name (must NOT compile)"; fail=1
  else
    err=$(grep -m1 "error" $NOUT/cf/$name.dbg.txt | head -c 120)
    echo "OK  correctly rejected: $name   [$err]"
  fi
done
[ $fail -eq 0 ] && echo "ALL COMPILE-FAIL PROBES PASS" || echo "COMPILE-FAIL PROBE FAILURE"
exit $fail
