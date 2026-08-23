#!/bin/bash
# Stage 3 — build every NROS crate (rlibs, unit-test binaries, demo bins, CLI, audit,
# studio server), the REAL macro→facade chain, examples, then compile & run all probes.
# All invocations below are the exact ones exercised during Pass 27.
set -u
T=${TOOLCHAIN_ROOT:-/home/user/toolchain}
NROS=${NROS_ROOT:-/home/user/NROS}
export RUSTC_VERSION=1.90.0 MRUSTC_TARGET_VER=1.90 OUTDIR_SUF=-1.90.0
export MINICARGO_DEFER_CODEGEN=0 PARLEVEL=2
M=$T/mrustc-master
MRUSTC=$M/bin/mrustc
STD=$M/output-1.90.0
NOUT=$T/nros-out
F=$T/facade-out
mkdir -p $NOUT $F
SELF=$(dirname "$(readlink -f "$0")")

mc() { # mc <src> <out> <crate-name> <crate-type> [extra args...]
  local src=$1 out=$2 name=$3 cty=$4; shift 4
  echo ">>> mrustc $name ($cty) <- $src"
  $MRUSTC "$src" -o "$out" --crate-name "$name" --crate-type "$cty" \
    --edition 2021 -O -L "$STD" -L "$NOUT" "$@" > "$out.dbg.txt" 2>&1
  local rc=$?
  if [ $rc -ne 0 ]; then echo "!!! FAIL($rc) $name"; tail -5 "$out.dbg.txt"; exit $rc; fi
}

echo "=== 3.1 rlibs ==="
mc $NROS/crates/nros-types/src/lib.rs       $NOUT/libnros_types.rlib       nros_types       rlib
mc $NROS/crates/nros-hal/src/lib.rs         $NOUT/libnros_hal.rlib         nros_hal         rlib
mc $NROS/crates/nros-sim/src/lib.rs         $NOUT/libnros_sim.rlib         nros_sim         rlib
mc $NROS/crates/nros-transport/src/lib.rs   $NOUT/libnros_transport.rlib   nros_transport   rlib
mc $NROS/crates/nros-distributed/src/lib.rs $NOUT/libnros_distributed.rlib nros_distributed rlib
mc $NROS/crates/nros-cli/src/lib.rs         $NOUT/libnros_cli.rlib         nros_cli         rlib
mc $NROS/crates/nros-studio/src/lib.rs      $NOUT/libnros_studio.rlib      nros_studio      rlib
mc $NROS/crates/nros-core/src/lib.rs        $NOUT/libnros_core.rlib        nros_core        rlib \
   --extern nros_types=$NOUT/libnros_types.rlib
mc $NROS/crates/nros-node/src/lib.rs        $NOUT/libnros_node.rlib        nros_node        rlib \
   --extern nros_types=$NOUT/libnros_types.rlib \
   --extern nros_core=$NOUT/libnros_core.rlib

echo "=== 3.2 unit-test binaries (--test) ==="
mc $NROS/crates/nros-types/src/lib.rs       $NOUT/test-nros_types       nros_types       bin --test
mc $NROS/crates/nros-hal/src/lib.rs         $NOUT/test-nros_hal         nros_hal         bin --test
mc $NROS/crates/nros-sim/src/lib.rs         $NOUT/test-nros_sim         nros_sim         bin --test
mc $NROS/crates/nros-transport/src/lib.rs   $NOUT/test-nros_transport   nros_transport   bin --test
mc $NROS/crates/nros-cli/src/lib.rs         $NOUT/test-nros_cli         nros_cli         bin --test
mc $NROS/crates/nros-studio/src/lib.rs      $NOUT/test-nros_studio      nros_studio      bin --test
mc $NROS/crates/nros-core/src/lib.rs        $NOUT/test-nros_core        nros_core        bin --test \
   --extern nros_types=$NOUT/libnros_types.rlib
# nros-node: explicit externs Required (trick #10)
mc $NROS/crates/nros-node/src/lib.rs        $NOUT/test-nros_node        nros_node        bin --test \
   --extern nros_types=$NOUT/libnros_types.rlib \
   --extern nros_core=$NOUT/libnros_core.rlib
# nros-distributed --test: mrustc typechecker crash (trick #11) — use dist-probe instead.

echo "=== 3.3 run unit suites ==="
for t in $NOUT/test-nros_*; do case "$t" in *.c|*.txt) continue;; esac; ./$t | tail -1; done

echo "=== 3.4 demo/other bins ==="
mc $NROS/crates/nros-core/src/main.rs        $NOUT/demo-core        demo_core        bin \
   --extern nros_types=$NOUT/libnros_types.rlib --extern nros_core=$NOUT/libnros_core.rlib
mc $NROS/crates/nros-core/src/bin/bench.rs   $NOUT/bench            bench            bin \
   --extern nros_types=$NOUT/libnros_types.rlib --extern nros_core=$NOUT/libnros_core.rlib
mc $NROS/crates/nros-node/src/main.rs        $NOUT/demo-node        demo_node        bin \
   --extern nros_types=$NOUT/libnros_types.rlib --extern nros_core=$NOUT/libnros_core.rlib \
   --extern nros_node=$NOUT/libnros_node.rlib
mc $NROS/crates/nros-hal/src/main.rs         $NOUT/demo-hal         demo_hal         bin \
   --extern nros_hal=$NOUT/libnros_hal.rlib
mc $NROS/crates/nros-sim/src/main.rs         $NOUT/demo-sim         demo_sim         bin \
   --extern nros_sim=$NOUT/libnros_sim.rlib
mc $NROS/crates/nros-transport/src/main.rs   $NOUT/demo-transport   demo_transport   bin \
   --extern nros_transport=$NOUT/libnros_transport.rlib
mc $NROS/crates/nros-distributed/src/main.rs $NOUT/demo-distributed demo_distributed bin \
   --extern nros_distributed=$NOUT/libnros_distributed.rlib
mc $NROS/crates/nros-cli/src/main.rs         $NOUT/nros-cli         nros_cli_bin     bin \
   --extern nros_cli=$NOUT/libnros_cli.rlib
mc $NROS/crates/nros-studio/src/main.rs      $NOUT/studio-server    nros_studio_bin  bin \
   --extern nros_studio=$NOUT/libnros_studio.rlib
mc $NROS/crates/nros-audit/src/main.rs       $NOUT/nros-audit       nros_audit_bin   bin
for d in demo-core demo-node demo-hal demo-sim demo-transport demo-distributed; do
  timeout 60 $NOUT/$d >/dev/null 2>&1 && echo "  ok $d" || { echo "  FAIL $d"; exit 1; }
done

echo "=== 3.5 golden templates (self-contained — trick #13) ==="
G=$T/golden; rm -rf $G; mkdir -p $G; cd $G
for tpl in basic mobile_base; do
  d=proj_$tpl; mkdir -p $d; (cd $d && $NOUT/nros-cli init proj_$tpl --template=$tpl >/dev/null 2>&1)
  $MRUSTC $G/proj_$tpl/proj_$tpl/src/main.rs -o $G/run_$tpl --crate-name golden_$tpl \
     --edition 2021 -O -L $STD >$G/$tpl.err 2>&1
  timeout 30 $G/run_$tpl >/dev/null 2>&1 && echo "  ok golden-$tpl" || { echo "  FAIL golden-$tpl"; exit 1; }
done
cd -

echo "=== 3.6 REAL macro chain + facade via minicargo (tricks #6,#7) ==="
SO=$M/script-overrides/stable-1.90.0-linux
printf 'cargo:rustc-cfg=wrap_proc_macro\n'                                     > $SO/build_proc-macro2.txt
:                                                                            > $SO/build_quote.txt
printf 'cargo:rustc-cfg=check_cfg\ncargo:rustc-cfg=syn_disable_nightly_tests\n' > $SO/build_syn.txt
$M/bin/minicargo --vendor-dir $T/vendor-macro --script-overrides $SO/ \
  --output-dir $F -L $STD -j 2 $NROS/crates/nros/

echo "=== 3.7 facade examples (real #[nros::node] expansion) ==="
for ex in mobile_base vertical_slice; do
  $MRUSTC $NROS/crates/nros/examples/$ex.rs -o $F/ex_$ex --crate-name ex_$ex --edition 2021 -O \
    -L $STD -L $F \
    --extern nros=$F/libnros.rlib \
    --extern nros_types=$F/libnros_types.rlib --extern nros_core=$F/libnros_core.rlib \
    --extern nros_node=$F/libnros_node.rlib --extern nros_hal=$F/libnros_hal.rlib \
    --extern nros_transport=$F/libnros_transport.rlib \
    --extern nros_distributed=$F/libnros_distributed.rlib \
    --extern nros_cli=$F/libnros_cli.rlib --extern nros_sim=$F/libnros_sim.rlib \
    --extern nros_studio=$F/libnros_studio.rlib \
    --extern nros_macros=$F/libnros_macros-plugin >$F/ex_$ex.err 2>&1
  timeout 60 $F/ex_$ex >/dev/null 2>&1 && echo "  ok example-$ex" || { echo "  FAIL example-$ex"; tail -3 $F/ex_$ex.err; exit 1; }
done

echo "=== 3.8 probes (compile + run) ==="
$MRUSTC $SELF/probes/ring-probe.rs -o $NOUT/ring-probe --crate-name ring_probe --edition 2021 -O \
  -L $STD -L $NOUT --extern nros_types=$NOUT/libnros_types.rlib --extern nros_core=$NOUT/libnros_core.rlib
$MRUSTC $SELF/probes/dist-probe.rs -o $NOUT/dist-probe --crate-name dist_probe --edition 2021 -O \
  -L $STD -L $NOUT --extern nros_distributed=$NOUT/libnros_distributed.rlib
$MRUSTC $SELF/probes/microbench.rs -o $NOUT/microbench --crate-name microbench --edition 2021 -O \
  -L $STD -L $NOUT --extern nros_types=$NOUT/libnros_types.rlib --extern nros_core=$NOUT/libnros_core.rlib
$MRUSTC $SELF/probes/fuzz-head.rs -o $NOUT/fuzz-head --crate-name fuzz_head --edition 2021 -O \
  -L $STD -L $NOUT --extern nros_transport=$NOUT/libnros_transport.rlib
$NOUT/ring-probe | tail -1
$NOUT/dist-probe | tail -1
$NOUT/microbench | tail -1
$NOUT/fuzz-head | tail -1
bash $SELF/probes/compile-fail.sh | tail -1
echo STAGE3_COMPLETE
