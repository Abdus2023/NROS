#!/bin/bash
# Stage 1 — bootstrap zlib + mrustc + the real rustc 1.90.0 source tree, offline-style.
# Network: ONLY github.com/codeload.github.com (no crates.io, no static.rust-lang.org).
# Reconstructed for backup from the Pass 27 live session (executed twice, both green).
set -euo pipefail
T=${TOOLCHAIN_ROOT:-/home/user/toolchain}
mkdir -p "$T/dl"; cd "$T"

export RUSTC_VERSION=1.90.0 MRUSTC_TARGET_VER=1.90 OUTDIR_SUF=-1.90.0
export MINICARGO_DEFER_CODEGEN=0 PARLEVEL=2
JOBS=${JOBS:-$(nproc)}

echo "=== stage1: zlib (mrustc build dependency) ==="
if [ ! -d "$T/zlib/lib" ]; then
  [ -e dl/zlib.tgz ] || curl -fL -o dl/zlib.tgz \
    https://codeload.github.com/madler/zlib/tar.gz/refs/tags/v1.3.1
  rm -rf zlib-1.3.1; tar xzf dl/zlib.tgz
  (cd zlib-1.3.1 && ./configure --prefix="$T/zlib" && make -j"$JOBS" && make install)
fi

echo "=== stage1: mrustc (master) ==="
if [ ! -d mrustc-master ]; then
  [ -e dl/mrustc.tgz ] || curl -fL -o dl/mrustc.tgz \
    https://codeload.github.com/thepowersgang/mrustc/tar.gz/refs/heads/master
  tar xzf dl/mrustc.tgz; mv mrustc-master mrustc-master
fi
cd mrustc-master
[ -e bin/mrustc ] || make -j"$JOBS" PARLEVEL=2 RUSTC_VERSION=1.90.0

echo "=== stage1: minicargo source patches (trick #4) ==="
python3 - <<'PY'
import sys
# 4a. manifest.cpp: tolerate unknown [workspace] / [workspace.package] keys.
p = "tools/minicargo/manifest.cpp"
s = open(p).read()
ok = True
def apply(s, old, new, tag):
    global ok
    if new in s:
        print(f"  already patched: {tag}"); return s
    if old not in s:
        print(f"  !! ANCHOR NOT FOUND (upstream drift?): {tag}"); ok = False; return s
    print(f"  patched: {tag}"); return s.replace(old, new, 1)

# NROS workspace manifests use [workspace.package] verbs (version/authors/description)
# that upstream minicargo rejects as unknown keys. Insert tolerant ignores in the two
# key-dispatch sites. (Anchor = the existing version_handler dispatch line.)
s = apply(s,
  'else if( key == "version" )',
  'else if( key == "version" || key == "authors" || key == "description" || key == "license" || key == "repository" )',
  "manifest.cpp workspace.package key tolerance")
s = apply(s,
  'else if( key == "members" )',
  'else if( key == "members" || key == "package" )',
  "manifest.cpp [workspace] package subsection tolerance")
open(p, "w").write(s)

# 4b. toml.h: as_string() must coerce Integer/Boolean to text (mutable buffer).
p2 = "tools/minicargo/toml.h"
s2 = open(p2).read()
def apply2(old, new, tag):
    global ok, s2
    if new in s2:
        print(f"  already patched: {tag}"); return
    if old not in s2:
        print(f"  !! ANCHOR NOT FOUND (upstream drift?): {tag}"); ok = False; return
    print(f"  patched: {tag}"); s2 = s2.replace(old, new, 1)
apply2("::std::string m_str_value;", "mutable ::std::string m_str_value;",
       "toml.h m_str_value mutable")
apply2("""    const ::std::string& as_string() const
    {
        if( m_type != Type::String )    throw ::std::runtime_error("TOML value not a string");
        return m_str_value;
    }""",
"""    const ::std::string& as_string() const
    {
        if( m_type == Type::String )     return m_str_value;
        // Pass 27: coerce scalar numerics/bools to text (workspace version fields
        // like `version = 1.99.0` arrive as Integer in minicargo's parser).
        if( m_type == Type::Integer ) { m_str_value = ::std::to_string(m_int_value); return m_str_value; }
        if( m_type == Type::Boolean ) { m_str_value = (m_bool_value ? "true" : "false"); return m_str_value; }
        throw ::std::runtime_error("TOML value not a string");
    }""",
    "toml.h as_string coercion")
open(p2, "w").write(s2)
sys.exit(0 if ok else 1)
PY
make -C tools/minicargo -j"$JOBS"

echo "=== stage1: rustc 1.90.0 source tree (trick #3: RUSTCSRC tarball) ==="
if [ ! -d rustc-1.90.0-src ]; then
  [ -e ../dl/rust-src.tgz ] || curl -fL -o ../dl/rust-src.tgz \
    https://codeload.github.com/rust-lang/rust/tar.gz/refs/tags/1.90.0
  rm -rf rust-1.90.0; tar xzf ../dl/rust-src.tgz
  mv rust-1.90.0 rustc-1.90.0-src          # top dir name MUST be rustc-1.90.0-src
fi
# The tarball lacks submodule contents; library/backtrace is a gitlink. Restore the
# exact pinned upstream commit (trick: gitlink -> backtrace-rs pin from Pass 27).
if ! grep -q "0.3.75" rustc-1.90.0-src/library/backtrace/Cargo.toml 2>/dev/null; then
  [ -e ../dl/backtrace.tgz ] || curl -fL -o ../dl/backtrace.tgz \
    https://codeload.github.com/rust-lang/backtrace-rs/tar.gz/b65ab935fb2e0d59dba8966ffca09c9cc5a5f57c
  tmp=$(mktemp -d); tar xzf ../dl/backtrace.tgz -C "$tmp"
  rm -rf rustc-1.90.0-src/library/backtrace
  mv "$tmp"/* rustc-1.90.0-src/library/backtrace; rm -rf "$tmp"
fi
tar czf rustc-1.90.0-src.tar.gz rustc-1.90.0-src/   # so `make RUSTCSRC` is satisfied
make RUSTCSRC RUSTC_VERSION=1.90.0

echo STAGE1_COMPLETE
