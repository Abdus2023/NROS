#!/bin/bash
# Stage 2 — assemble the vendored std dependency tree (+ in-tree copies) and the macro
# chain, entirely from GitHub tag tarballs (no crates.io).
# Version TRUTH = rustc-1.90.0-src/library/Cargo.lock (printed at start for review).
set -euo pipefail
T=${TOOLCHAIN_ROOT:-/home/user/toolchain}
M=$T/mrustc-master
R=$M/rustc-1.90.0-src
D=$T/dl

# Pass 28: make the locally built zlib visible to EVERY gcc/g++ invocation, not just
# the one in stage 1. mrustc's src/memory_dump.cpp includes <zlib.h> and its Makefile
# links -lz; `make -f minicargo.mk` (stage 2) re-links bin/mrustc through a recursive
# make that does not inherit stage 1's command-line LINKFLAGS, so it dies with
# "cannot find -lz" unless the toolchain search path is set in the environment.
# CPATH / LIBRARY_PATH are honoured by gcc for compile and link; LD_LIBRARY_PATH is
# needed at run time because that re-link does not carry an -rpath.
if [ ! -e /usr/include/zlib.h ]; then
  export CPATH="$T/zlib/include${CPATH:+:$CPATH}"
  export LIBRARY_PATH="$T/zlib/lib${LIBRARY_PATH:+:$LIBRARY_PATH}"
  export LD_LIBRARY_PATH="$T/zlib/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
V=$R/vendor
mkdir -p "$D"
export RUSTC_VERSION=1.90.0 MRUSTC_TARGET_VER=1.90 OUTDIR_SUF=-1.90.0

echo "=== lockfile truth (review against the table below) ==="
grep -A1 '^name = "' $R/library/Cargo.lock | grep -E "^name|^version" | paste - - | head -40

# ── Download table ──────────────────────────────────────────────────────────────
# name=FILE_STEM codeload_path REF   (tag names verified during Pass 27; if a tag 404s,
# check `git ls-remote --tags https://github.com/<repo>` for the tag spelling)
fetch() { # fetch <stem> <codeload-url>
  [ -e "$D/$1.tgz" ] || curl -fL -o "$D/$1.tgz" "$2"
}

# std vendor set.
# Pass 28: re-derived from the ACTUAL library/Cargo.lock inside the rust 1.90.0 tree
# this script downloads. The previous table did not match that lockfile, and four of
# its URLs were dead (every one of those is noted below). Corrections:
#   - allocator-api2 / foldhash / equivalent are NOT in the 1.90.0 lockfile. std
#     depends on `hashbrown = { version = "0.15", default-features = false,
#     features = ['rustc-dep-of-std'] }`, and the lockfile's hashbrown 0.15.4 entry
#     depends only on rustc-std-workspace-{alloc,core} — so hashbrown's default-hasher
#     (foldhash) and allocator-api2 deps are switched off and must not be vendored.
#     Both pinned allocator-api2 URLs are dead anyway: GoldsteinE/allocator-api2 does
#     not exist, and tkaitchuck/ahash has no `allocator-api2-*` tag. (This is what
#     aborted stage 2 with two curl 404s.)
#   - adler2 has no tags upstream at all; pinned by commit whose Cargo.toml declares
#     version = "2.0.1" (matches the lockfile).
#   - rustc-demangle tags carry no `v` prefix, and 0.1.25 was never tagged — the
#     lockfile's 0.1.25 is unreachable from GitHub, so pin the 0.1.24 tag, which
#     satisfies std's `rustc-demangle = "0.1.24"` requirement.
#   - addr2line/gimli/object/memchr were pinned one or more minor versions behind the
#     1.90.0 lockfile (0.24.2/0.31.1/0.36.7/2.7.6 vs 0.25.0/0.32.0/0.37.1/2.7.5).
#   - the lockfile pins exactly one unicode-width (0.2.1, required by getopts), so
#     only one copy is vendored.
fetch libc              https://codeload.github.com/rust-lang/libc/tar.gz/refs/tags/0.2.174
fetch cfgif             https://codeload.github.com/rust-lang/cfg-if/tar.gz/refs/tags/v1.0.1
fetch hashbrown         https://codeload.github.com/rust-lang/hashbrown/tar.gz/refs/tags/v0.15.4
fetch addr2line         https://codeload.github.com/gimli-rs/addr2line/tar.gz/refs/tags/0.25.0
fetch gimli             https://codeload.github.com/gimli-rs/gimli/tar.gz/refs/tags/0.32.0
fetch object            https://codeload.github.com/gimli-rs/object/tar.gz/refs/tags/0.37.1
fetch miniz             https://codeload.github.com/Frommi/miniz_oxide/tar.gz/refs/tags/0.8.9
fetch adler2            https://codeload.github.com/oyvindln/adler2/tar.gz/89a031a0f42eeff31c70dc598b398cbf31f1680f
fetch memchr            https://codeload.github.com/BurntSushi/memchr/tar.gz/refs/tags/2.7.5
fetch getopts           https://codeload.github.com/rust-lang/getopts/tar.gz/refs/tags/v0.2.23
fetch uwidth            https://codeload.github.com/unicode-rs/unicode-width/tar.gz/refs/tags/v0.2.1
fetch demangle          https://codeload.github.com/rust-lang/rustc-demangle/tar.gz/refs/tags/0.1.24
fetch backtrace         https://codeload.github.com/rust-lang/backtrace-rs/tar.gz/b65ab935fb2e0d59dba8966ffca09c9cc5a5f57c

# macro chain (real sources; strip [patch.*] sections — minicargo TODO on those)
fetch pm2               https://codeload.github.com/dtolnay/proc-macro2/tar.gz/refs/tags/1.0.107
fetch quote             https://codeload.github.com/dtolnay/quote/tar.gz/refs/tags/1.0.47
fetch syn               https://codeload.github.com/dtolnay/syn/tar.gz/refs/tags/2.0.119
fetch uid               https://codeload.github.com/dtolnay/unicode-ident/tar.gz/refs/tags/1.0.24

echo "=== assemble vendor/ ==="
rm -rf "$V"; mkdir -p "$V"
put() { # put <stem> <target-dir-name>
  local tmp; tmp=$(mktemp -d)
  tar xzf "$D/$1.tgz" -C "$tmp"
  rm -rf "$V/$2"; mv "$tmp"/* "$V/$2"; rm -rf "$tmp"
  echo "  $2"
}
put libc libc
put cfgif cfg-if
put hashbrown hashbrown
put addr2line addr2line
put gimli gimli
# Pass 28: `object` and `adler2` were downloaded by the fetch table above but never
# copied into vendor/, so minicargo could not resolve std's `backtrace` feature
# (object) or miniz_oxide's dependency (adler2).
put object object
put adler2 adler2
put memchr memchr
put getopts getopts
put uwidth unicode-width
put demangle rustc-demangle
# NOTE: some GitHub tag tarballs hold a workspace; if a put() lands you a metadata dir
# instead of the crate (no src/), descend to the inner crate subdir before moving on.
for c in $V/*/; do [ -d "$c/src" ] || echo "  !! review $c (workspace tarball?)"; done

# miniz_oxide: the crate is the inner miniz_oxide/ subdir
tmp=$(mktemp -d); tar xzf $D/miniz.tgz -C $tmp
rm -rf $V/miniz_oxide; mv $tmp/*/miniz_oxide $V/miniz_oxide; rm -rf $tmp
echo "  miniz_oxide (inner subdir)"

echo "=== in-tree copies ==="
# Pass 28: do NOT copy compiler-builtins or libm into vendor/.
# Both are reachable from the std graph as *path* dependencies
# (library/alloc/Cargo.toml: `compiler_builtins = { path =
# "../compiler-builtins/compiler-builtins", features = ["rustc-dep-of-std"] }`), so a
# second copy in vendor/ gives minicargo two packages named `compiler_builtins 0.1.160`.
# It then builds one and references the other: the build ran
#   mrustc rustc-1.90.0-src/vendor/compiler_builtins/src/lib.rs --crate-tag 0_1_160_H22
# while library/alloc was invoked with
#   --extern compiler_builtins=output-1.90.0/libcompiler_builtins-0_1_160_Ha2.rlib
# and died with "Unable to open crate 'compiler_builtins'". (Trick #5's build-ordering
# hazard — compiler_builtins scheduled before libcore.rlib exists — only arises with
# parallel jobs; this stage now builds serially.)
# Pass 28 (corrected): the vendored compiler_builtins copy IS required —
# library/std/Cargo.toml declares `compiler_builtins = { version = "0.1.2", ... }`, a
# *registry* dependency that minicargo can only resolve from vendor/. What must NOT
# happen is editing its feature lists: library/alloc reaches the same package through
# a *path* dependency with `features = ["rustc-dep-of-std"]`, and minicargo derives the
# crate tag (the `Hxx` suffix on the rlib filename) from the feature set. Rewriting
# `default` / `rustc-dep-of-std` in the vendored copy made the built tag (H22) differ
# from the one alloc asked for (Ha2) -> "Unable to open crate 'compiler_builtins'".
# So: copy it, re-anchor its `core` path, and leave the feature lists exactly as
# upstream has them.
cp -a $R/library/compiler-builtins/compiler-builtins $V/compiler_builtins
# libm is NOT a dependency crate here -- compiler_builtins' src/math/mod.rs pulls it in
# by source path (`#[path = "../../libm/src/libm_math.rs"]`), so it must sit next to the
# vendored copy or mrustc fails with "Can't find file for 'libm_math'".
cp -a $R/library/compiler-builtins/libm $V/libm
cp -a $R/library/rustc-std-workspace-alloc $V/rustc-std-workspace-alloc
cp -a $R/library/rustc-std-workspace-core $V/rustc-std-workspace-core
cp -a $R/library/rustc-std-workspace-std $V/rustc-std-workspace-std

echo "=== path fixes for in-tree copies (../X references break inside vendor/) ==="
# Pass 28: rewritten. The previous three sed expressions did not match the 1.90.0 tree:
#   - compiler_builtins declares `core = { path = "../../../library/core" }` (upstream
#     the crate sits at library/compiler-builtins/compiler-builtins/), which from
#     vendor/ resolves OUTSIDE the rust tree;
#   - rustc-std-workspace-alloc/std declare `path = "../alloc"` / `path = "../std"`,
#     which from vendor/ resolves to the non-existent vendor/alloc and vendor/std;
#   - rustc-std-workspace-core declares `path = "../compiler-builtins/compiler-builtins"`,
#     which the old `s|path = "\.\./compiler-builtins"|` anchor never matched (the
#     closing quote is not adjacent), so that rewrite was a silent no-op.
# Rewrite by *crate name* instead of by literal source string, then verify on disk
# that every remaining path dependency actually resolves. Silent no-op rewrites are
# exactly what made the old block fail without saying why.
python3 - "$V" "$R" <<'PYEOF'
import os, re, sys
V, R = sys.argv[1], sys.argv[2]
LIB = os.path.join(R, "library")
MAP = {
    "core": os.path.join(LIB, "core"),
    "alloc": os.path.join(LIB, "alloc"),
    "std": os.path.join(LIB, "std"),
    "test": os.path.join(LIB, "test"),
    "panic_unwind": os.path.join(LIB, "panic_unwind"),
    "panic_abort": os.path.join(LIB, "panic_abort"),
    "libm": os.path.join(LIB, "compiler-builtins", "libm"),
    # Pass 28: point at the IN-TREE compiler-builtins crate. It is no longer copied
    # into vendor/ (see the note above), and keeping a single definition is the whole
    # point — two copies named `compiler_builtins 0.1.160` make minicargo build one
    # and link the other.
    "compiler_builtins": os.path.join(LIB, "compiler-builtins", "compiler-builtins"),
}
CRATES = [c for c in os.listdir(V)
          if os.path.exists(os.path.join(V, c, "Cargo.toml"))
          and (c.startswith("rustc-std-workspace-") or c in ("compiler_builtins", "libm"))]
rewritten = 0
for c in CRATES:
    p = os.path.join(V, c, "Cargo.toml")
    if not os.path.exists(p):
        print(f"  !! missing {p}"); sys.exit(1)
    crate_dir = os.path.dirname(p)
    s = open(p).read()
    def repl(m):
        global rewritten
        rel = m.group(1)
        if rel.endswith(".rs"):
            return m.group(0)
        key = rel.rstrip("/").split("/")[-1].replace("-", "_")
        if key not in MAP:
            return m.group(0)
        new = os.path.relpath(MAP[key], crate_dir)
        if new != rel:
            rewritten += 1
        return 'path = "%s"' % new
    open(p, "w").write(re.sub(r'path = "([^"]+)"', repl, s))

# Verification: every remaining path dependency must exist on disk.
bad = []
for c in CRATES:
    p = os.path.join(V, c, "Cargo.toml")
    crate_dir = os.path.dirname(p)
    for m in re.finditer(r'path = "([^"]+)"', open(p).read()):
        rel = m.group(1)
        if rel.endswith(".rs"):
            continue
        if not os.path.isdir(os.path.normpath(os.path.join(crate_dir, rel))):
            bad.append(f'{c}: path = "{rel}"')
print(f"  rewrote {rewritten} path dependency reference(s)")
for b in sorted(set(bad)):
    print(f"  !! UNRESOLVED {b}")
if bad:
    sys.exit(1)
print("  all in-tree path dependencies resolve")
PYEOF

# compiler_builtins: DECLARE a non-optional path dep on the real in-tree core (trick #5).
# Removing it (the first working recipe) leaves minicargo with no dependency edge and
# can schedule compiler_builtins before libcore.rlib exists -> "Unable to locate crate
# 'core'" abort at t=0. The edge is semantically neutral (only #[cfg(test)] extern crate
# core exists in sources). Also default features must avoid asm (mrustc emits C; no
# naked fns), which the overrides file also adds on the workspace-core side.
if [ -f "$V/compiler_builtins/Cargo.toml" ]; then
NROS_CB_INTREE="$R/library/compiler-builtins/compiler-builtins/Cargo.toml" \
python3 - "$V/compiler_builtins/Cargo.toml" <<'PYEOF'
import os, re, sys
p = sys.argv[1]
s = open(p).read()
# (a) Re-anchor the `core` path: the crate moved from
#     library/compiler-builtins/compiler-builtins/ into vendor/, so the upstream
#     ../../../library/core no longer resolves from there.
# (b) Make the `core` dependency NON-optional (trick #5). Without this, minicargo
#     passes no `--extern core=...` on this edge -- `core` is optional and is only
#     pulled in by the `rustc-dep-of-std` feature, which is not enabled here -- and
#     mrustc aborts with "Unable to locate crate 'core' in search directories".
#     The edge is semantically neutral (only `#[cfg(test)] extern crate core`).
# Deliberately does NOT touch the [features] lists: the feature set is what the rlib
# crate tag (the `Hxx` suffix) is derived from, and library/alloc reaches this same
# package through a path dep with `features = ["rustc-dep-of-std"]`. Rewriting
# `default` / `rustc-dep-of-std` here made the built tag differ from the one alloc
# asked for, failing with "Unable to open crate 'compiler_builtins'".
s2 = re.sub(r'(?m)^core = \{ path = "[^"]+", optional = true \}$',
            'core = { path = "../../library/core" }', s)
if not re.search(r'(?m)^core = \{ path = "\.\./\.\./library/core" \}$', s2):
    print("  !! compiler_builtins core dep is not a non-optional ../../library/core edge")
    print("     got: " + next((l for l in s2.splitlines() if l.startswith("core =")), "<absent>"))
    sys.exit(1)
# (c) Add `no-asm` to `default`, and do it in BOTH the vendored copy and the in-tree
#     crate so the two manifests agree. compiler_builtins' x86 routines use inline
#     asm; mrustc emits C, and the assembler then dies with
#       Error: previous CFI entry not closed (missing .cfi_endproc)
#       Error: .cfi_endproc without corresponding .cfi_startproc
#     The `no-asm` feature switches those to the pure-Rust implementations. It has to
#     be on `default` (not just on one dependency edge) because the crate is reached
#     from two different edges -- library/alloc with `rustc-dep-of-std` and
#     library/rustc-std-workspace-core with `compiler-builtins` -- and a per-edge
#     feature addition leaves the built variant and the referenced variant with
#     different feature sets, i.e. different crate tags.
s3 = re.sub(r'(?m)^default = \[.*\]$', 'default = ["compiler-builtins", "no-asm"]', s2)
open(p, "w").write(s3)
intree = os.environ["NROS_CB_INTREE"]
t = open(intree).read()
t2 = re.sub(r'(?m)^default = \[.*\]$', 'default = ["compiler-builtins", "no-asm"]', t)
if t2 != t:
    open(intree, "w").write(t2)
    print("  in-tree compiler-builtins default features matched (no-asm)")
print("  compiler_builtins: core re-anchored + non-optional, default += no-asm (both copies)")
PYEOF
else
  echo "  (no vendored compiler_builtins copy — in-tree crate is used directly)"
fi

echo "=== library/backtrace restored to pinned upstream commit ==="
if ! grep -q '0\.3\.75' $R/library/backtrace/Cargo.toml 2>/dev/null; then
  tmp=$(mktemp -d); tar xzf $D/backtrace.tgz -C $tmp
  rm -rf $R/library/backtrace
  mv $tmp/* $R/library/backtrace
  rm -rf $tmp
fi
ls $R/library/backtrace/src | head -3

echo "=== macro vendor chain (separate dir) ==="
T2=$T/vendor-macro
mkdir -p $T2
for c in pm2:proc-macro2 quote:quote syn:syn uid:unicode-ident; do
  tgz=${c%%:*}; name=${c##*:}
  tmp=$(mktemp -d); tar xzf $D/$tgz.tgz -C $tmp
  rm -rf $T2/$name; mv $tmp/* $T2/$name; rm -rf $tmp
  # strip [patch.*] sections (minicargo TODO on repository patches)
  python3 - "$T2/$name/Cargo.toml" <<'PYEOF'
import re, sys
p = sys.argv[1]
s = open(p).read()
s2 = re.sub(r"(?ms)^\[patch\.[^\n]*\n(?:[^\[]*\n|\n)*?(?=^\[|\Z)", "", s)
open(p, "w").write(s2)
PYEOF
done
ls $T2

echo "=== build std (make LIBS) ==="
cd $M
# Pass 28: do NOT hand make a parallel job count here. `make -j2` exports a jobserver
# through MAKEFLAGS, and minicargo (tools/minicargo/jobs.cpp: `num_jobs == 0` ->
# `JobServer::create(0)`) attaches to it as a client, so the old `-j2` silently let it
# run two mrustc invocations at once. On the rust 1.90.0 `mrustc-stdlib/` graph
# minicargo schedules `core` twice (once via a relative and once via an absolute
# manifest path), and two concurrent mrustc+gcc runs over libcore's ~37 MB of
# generated C exhaust a small sandbox — the build died with
# "Process was terminated with signal 9" (OOM kill) on `core`.
# Serial by default; raise with `JOBS=2 bash stage2-...` only on a box with the RAM.
make -f minicargo.mk LIBS -j"${JOBS:-1}"
ls output-1.90.0/libstd.rlib && echo STAGE2_COMPLETE
