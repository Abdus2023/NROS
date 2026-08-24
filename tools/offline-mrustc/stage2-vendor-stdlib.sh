#!/bin/bash
# Stage 2 — assemble the vendored std dependency tree (+ in-tree copies) and the macro
# chain, entirely from GitHub tag tarballs (no crates.io).
# Version TRUTH = rustc-1.90.0-src/library/Cargo.lock (printed at start for review).
set -euo pipefail
T=${TOOLCHAIN_ROOT:-/home/user/toolchain}
M=$T/mrustc-master
R=$M/rustc-1.90.0-src
D=$T/dl
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

# std vendor set (from the real library/Cargo.lock of rust 1.90.0)
fetch libc              https://codeload.github.com/rust-lang/libc/tar.gz/refs/tags/0.2.174
fetch cfgif             https://codeload.github.com/rust-lang/cfg-if/tar.gz/refs/tags/v1.0.1
fetch hashbrown         https://codeload.github.com/rust-lang/hashbrown/tar.gz/refs/tags/v0.15.4
fetch allocapi2         https://codeload.github.com/tkaitchuck/ahash/tar.gz/refs/tags/allocator-api2-v0.2.21 || \
  curl -fL -o "$D/allocapi2.tgz" https://codeload.github.com/GoldsteinE/allocator-api2/tar.gz/refs/tags/v0.2.21
fetch foldhash          https://codeload.github.com/orlp/foldhash/tar.gz/refs/tags/v0.1.5
fetch equivalent        https://codeload.github.com/indexmap-rs/equivalent/tar.gz/refs/tags/1.0.2
fetch addr2line         https://codeload.github.com/gimli-rs/addr2line/tar.gz/refs/tags/0.24.2
fetch gimli             https://codeload.github.com/gimli-rs/gimli/tar.gz/refs/tags/0.31.1
fetch object            https://codeload.github.com/gimli-rs/object/tar.gz/refs/tags/0.36.7
fetch miniz             https://codeload.github.com/Frommi/miniz_oxide/tar.gz/refs/tags/0.8.9
fetch adler2            https://codeload.github.com/oyvindln/adler2/tar.gz/refs/tags/v2.0.1
fetch memchr            https://codeload.github.com/BurntSushi/memchr/tar.gz/refs/tags/2.7.6
fetch getopts           https://codeload.github.com/rust-lang/getopts/tar.gz/refs/tags/v0.2.23
fetch uwidth1           https://codeload.github.com/unicode-rs/unicode-width/tar.gz/refs/tags/v0.1.14
fetch uwidth2           https://codeload.github.com/unicode-rs/unicode-width/tar.gz/refs/tags/v0.2.1
fetch demangle          https://codeload.github.com/rust-lang/rustc-demangle/tar.gz/refs/tags/v0.1.24
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
put allocapi2 allocator-api2
put foldhash foldhash
put equivalent equivalent
put addr2line addr2line
put gimli gimli
put memchr memchr
put getopts getopts
put uwidth1 unicode-width
put uwidth2 unicode-width-0.2
put demangle rustc-demangle
# NOTE: some GitHub tag tarballs hold a workspace; if a put() lands you a metadata dir
# instead of the crate (no src/), descend to the inner crate subdir before moving on.
for c in $V/*/; do [ -d "$c/src" ] || echo "  !! review $c (workspace tarball?)"; done

# miniz_oxide: the crate is the inner miniz_oxide/ subdir
tmp=$(mktemp -d); tar xzf $D/miniz.tgz -C $tmp
rm -rf $V/miniz_oxide; mv $tmp/*/miniz_oxide $V/miniz_oxide; rm -rf $tmp
echo "  miniz_oxide (inner subdir)"

echo "=== in-tree copies ==="
cp -a $R/library/compiler-builtins/compiler-builtins $V/compiler_builtins
cp -a $R/library/compiler-builtins/libm $V/libm
cp -a $R/library/rustc-std-workspace-alloc $V/rustc-std-workspace-alloc
cp -a $R/library/rustc-std-workspace-core $V/rustc-std-workspace-core
cp -a $R/library/rustc-std-workspace-std $V/rustc-std-workspace-std

echo "=== path fixes for in-tree copies (../X references break inside vendor/) ==="
# rustc-std-workspace crates + compiler_builtins reference sibling library paths
for f in $V/rustc-std-workspace-*/Cargo.toml $V/compiler_builtins/Cargo.toml; do
  [ -f "$f" ] || continue
  sed -i \
    -e 's|path = "\.\./core"|path = "../../library/core"|g' \
    -e 's|path = "\.\./\.\./core"|path = "../../../library/core"|g' \
    -e 's|path = "\.\./compiler-builtins"|path = "../../library/compiler-builtins/compiler-builtins"|g' \
    "$f"
done

# compiler_builtins: DECLARE a non-optional path dep on the real in-tree core (trick #5).
# Removing it (the first working recipe) leaves minicargo with no dependency edge and
# can schedule compiler_builtins before libcore.rlib exists -> "Unable to locate crate
# 'core'" abort at t=0. The edge is semantically neutral (only #[cfg(test)] extern crate
# core exists in sources). Also default features must avoid asm (mrustc emits C; no
# naked fns), which the overrides file also adds on the workspace-core side.
python3 - "$V/compiler_builtins/Cargo.toml" <<'PYEOF'
import re, sys
p = sys.argv[1]
s = open(p).read()
# Rewrite the optional `core` path dependency to the real library/core, non-optional
s = re.sub(r'(?ms)^core = \{ path = "\.\./\.\./core", optional = true \}\n',
           'core = { path = "../../library/core" }\n', s)
s = s.replace('path = "../../core"', 'path = "../../library/core"')
# ensure default features avoid asm
s = re.sub(r'(?m)^default = \[.*\]$', 'default = ["compiler-builtins", "no-asm"]', s)
# ensure rustc-dep-of-std pulls compiler-builtins
if '[features]' in s and 'rustc-dep-of-std' in s:
    s = re.sub(r'(?m)^rustc-dep-of-std = \[.*\]$', 'rustc-dep-of-std = ["compiler-builtins"]', s)
assert 'core = { path = "../../library/core" }' in s, "core edge must be present"
open(p, "w").write(s)
print("compiler_builtins manifest patched (non-optional core edge, no-asm default)")
PYEOF

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
make -f minicargo.mk LIBS -j2
ls output-1.90.0/libstd.rlib && echo STAGE2_COMPLETE
