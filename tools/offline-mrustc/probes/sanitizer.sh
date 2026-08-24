#!/bin/bash
# Sanitizer pass — rebuild the NROS unit-test binaries with AddressSanitizer +
# UndefinedBehaviorSanitizer and run them.
#
# Why this exists: Miri is the only UB detector wired into CI and it has never produced
# a passing result, so nros-core's unsafe code (MaybeUninit, drop_in_place, raw pointer
# arithmetic over a shared ring) had no executed memory-safety evidence at all. mrustc
# emits C, which means the generated sources can simply be recompiled with gcc's
# sanitizers — no Rust toolchain required.
#
# What this DOES catch: heap-buffer-overflow, use-after-free, double-free,
# use-after-return/scope, and the UBSan checks (misaligned pointers, signed overflow,
# null deref, invalid shifts, ...).
#
# What this does NOT catch, and therefore is NOT a substitute for Miri:
#   - reading uninitialized memory (that is MemorySanitizer, which needs every object
#     file including libstd and an instrumented libc — not practical here)
#   - Rust-specific UB that has no C analogue: creating an invalid reference or `&mut`
#     alias, an invalid enum discriminant, violating `Pin`, `transmute` misuse
# So a clean run here narrows the risk; it does not close the Miri gap.
#
# Usage: bash probes/sanitizer.sh [nros-out-dir]
set -u

T=${TOOLCHAIN_ROOT:-/home/user/.cache/nros-toolchain}
NOUT=${1:-$T/nros-out}
STD=$T/mrustc-master/output-1.90.0
export LD_LIBRARY_PATH="$T/zlib/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

[ -d "$NOUT" ] || { echo "no such output dir: $NOUT (run stage3 first)"; exit 2; }
command -v gcc >/dev/null || { echo "gcc not found"; exit 2; }

# --- 0. capability check -------------------------------------------------------
printf 'int main(){return 0;}\n' > /tmp/_san_probe.c
if ! gcc -fsanitize=address,undefined -g /tmp/_san_probe.c -o /tmp/_san_probe 2>/dev/null; then
  echo "SANITIZERS UNAVAILABLE (gcc lacks libasan/libubsan) — skipping"
  exit 0
fi
rm -f /tmp/_san_probe /tmp/_san_probe.c

# --- 1. rebuild every test binary from mrustc's generated C with sanitizers ----
built=0
for cmd in "$NOUT"/test-nros_*_cmd.txt; do
  [ -e "$cmd" ] || continue
  name=$(basename "$cmd" _cmd.txt)
  python3 - "$cmd" "$NOUT/$name-asan" <<'PY'
import shlex, subprocess, sys
cmdfile, outpath = sys.argv[1], sys.argv[2]
args = shlex.split(open(cmdfile).read())
out, i = [], 0
while i < len(args):
    if args[i] == "-o":
        out += ["-o", outpath]; i += 2; continue
    out.append(args[i]); i += 1
sys.exit(subprocess.call(
    ["gcc", "-fsanitize=address,undefined", "-fno-sanitize-recover=all",
     "-fno-omit-frame-pointer", "-g"] + out))
PY
  if [ $? -eq 0 ]; then built=$((built+1)); else echo "  !! failed to build $name-asan"; exit 1; fi
done
echo "built $built sanitized test binary/binaries"

# --- 2. run them ---------------------------------------------------------------
# Leak detection is OFF for the pass/fail verdict. mrustc's libstd port leaks a fixed
# amount at exit from std::thread::Thread::new plus runtime startup, in every binary
# regardless of the code under test; measured baselines are printed in step 3 so the
# distinction stays visible instead of being silently ignored.
export ASAN_OPTIONS=detect_leaks=0
export UBSAN_OPTIONS=print_stacktrace=1
fail=0
for bin in "$NOUT"/test-nros_*-asan; do
  [ -x "$bin" ] || continue
  name=$(basename "$bin")
  out=$(timeout 600 "$bin" 2>&1); rc=$?
  reports=$(printf '%s' "$out" | grep -cE "AddressSanitizer:|runtime error")
  result=$(printf '%s' "$out" | grep -E "test result" | tail -1)
  if [ "$rc" -eq 0 ] && [ "$reports" -eq 0 ]; then
    printf "  CLEAN %-22s %s\n" "$name" "$result"
  else
    printf "  DIRTY %-22s rc=%s reports=%s\n" "$name" "$rc" "$reports"
    printf '%s\n' "$out" | grep -E "AddressSanitizer:|runtime error" | head -5
    fail=1
  fi
done

# --- 3. leak baseline (informational) -----------------------------------------
# Run one binary WITH leak detection and show where the frames come from. If no frame
# mentions the crate under test, the leak belongs to the runtime, not to NROS.
export ASAN_OPTIONS=detect_leaks=1:fast_unwind_on_malloc=0:malloc_context_size=40
probe="$NOUT/test-nros_core-asan"
if [ -x "$probe" ]; then
  echo "  --- leak attribution for test-nros_core-asan (informational) ---"
  timeout 600 "$probe" > /tmp/_san_leak.log 2>&1
  total=$(grep -oE "SUMMARY: AddressSanitizer: [0-9]+ byte" /tmp/_san_leak.log | grep -oE "[0-9]+")
  echo "      leaked bytes at exit: ${total:-0}"

  # mrustc frames look like either
  #     "#N 0xADDR in <symbol> /abs/path/test-nros_core.c:LINE"
  # or  "#N 0xADDR in <symbol> (/abs/path/test-nros_core-asan+0xOFF)"
  # BOTH tails contain the crate name, so grepping the raw line matches every frame
  # and reports a meaningless count. Extract the symbol only. Note also that this
  # binary's crate mangles as `bin`, not `nros_core`, so the crate name is not a
  # usable symbol match here at all — RingBuffer is.
  syms=$(grep -E "^    #[0-9]+ " /tmp/_san_leak.log \
           | sed -E 's/^    #[0-9]+ 0x[0-9a-f]+ in //; s/[ (].*$//')
  rb=$(printf '%s\n' "$syms" | grep -c "RingBuffer" || true)
  echo "      leak frames whose SYMBOL names RingBuffer: ${rb:-0}"
  echo "      test functions appearing in leak stacks:"
  printf '%s\n' "$syms" | grep -oE "tests[0-9]+[a-z_]+" | sed -E 's/tests[0-9]+//' \
    | grep -E '^.{4,}$' | sort -u | sed 's/^/        /'
  echo "      deepest recurring frames:"
  printf '%s\n' "$syms" | sort | uniq -c | sort -rn | head -4 | sed 's/^/        /'
fi

if [ "$fail" -eq 0 ]; then
  echo "ALL SANITIZER PROBES CLEAN (ASan+UBSan; leak detection excluded, see above)"
else
  echo "SANITIZER PROBES REPORTED ERRORS"
  exit 1
fi
