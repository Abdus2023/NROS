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
  # Pass 28 fix: the codeload tarball already extracts to `mrustc-master/`, so the
  # previous `mv mrustc-master mrustc-master` was a self-move that fails under
  # `set -e` ("cannot move ... to a subdirectory of itself") and aborted stage 1 on
  # every clean run. Extract and assert instead.
  tar xzf dl/mrustc.tgz
fi
[ -f mrustc-master/Makefile ] || { echo "!! mrustc-master/Makefile missing after extract"; exit 1; }
cd mrustc-master
# Pass 28 fix: mrustc's src/memory_dump.cpp does `#include <zlib.h>` and its Makefile
# links `-lz`, but neither the compiler nor the linker is told where the zlib that
# stage 1 just built lives. On a sandbox without system zlib-dev this fails with
# "fatal error: zlib.h: No such file or directory". Feed the local zlib through the
# Makefile's own CXXFLAGS_EXTRA / LINKFLAGS hooks (command-line vars override the
# Makefile's `:=` assignments).
MRUSTC_MAKE_ARGS=()
if [ ! -e /usr/include/zlib.h ]; then
  echo "  no system zlib.h -> using locally built zlib at $T/zlib"
  MRUSTC_MAKE_ARGS=(
    "CXXFLAGS_EXTRA=-I$T/zlib/include"
    "LINKFLAGS=-g -L$T/zlib/lib -Wl,-rpath,$T/zlib/lib"
  )
fi
[ -e bin/mrustc ] || make -j"$JOBS" PARLEVEL=2 RUSTC_VERSION=1.90.0 "${MRUSTC_MAKE_ARGS[@]}"

echo "=== stage1: minicargo source patches (trick #4) ==="
python3 - <<'PY'
import os, sys

ok = True
def apply(path, old, new, tag):
    """Apply one anchored source patch; loud on drift, idempotent on re-run."""
    global ok
    if not os.path.exists(path):
        print(f"  !! FILE MISSING (upstream drift?): {path} [{tag}]"); ok = False; return
    s = open(path).read()
    if new in s:
        print(f"  already patched: {tag}"); return
    if old not in s:
        print(f"  !! ANCHOR NOT FOUND (upstream drift?): {tag}"); ok = False; return
    open(path, "w").write(s.replace(old, new, 1))
    print(f"  patched: {tag}")

# 4a. tools/minicargo/manifest.cpp — accept the [workspace.package] metadata keys that
# NROS inherits (`version`, `authors`, `description`, ...). mrustc master already
# tolerates `members`/`exclude`/`resolver`/`dependencies` and, inside
# `[workspace.package]`, `edition`/`rust-version`/`license`/`homepage`/`repository`,
# but hard-errors on anything else — which includes NROS's `version`/`authors`/
# `description`. (Pass 28: the previous anchors targeted `key == "version"` and
# `key == "members"`, which no longer exist in this form upstream.)
apply("tools/minicargo/manifest.cpp",
"""                else {
                    eh.error("Unknown item in [workspace.package] : `", key, "`");
                }""",
"""                else if( key == "version" || key == "authors" || key == "description"
                      || key == "keywords" || key == "readme" || key == "documentation"
                      || key == "categories" || key == "publish" || key == "include"
                      || key == "exclude" ) {
                    // Pass 28: inherited package metadata. minicargo needs no value
                    // for these, it only must not reject the manifest.
                }
                else {
                    eh.error("Unknown item in [workspace.package] : `", key, "`");
                }""",
"manifest.cpp [workspace.package] metadata key tolerance")

# 4b. as_string() must coerce Integer/Boolean to text. mrustc master moved the TOML
# reader from tools/minicargo/toml.h to tools/common/toml.h, renamed the failure to
# `TypeError`, and stores booleans in m_int_value (there is no m_bool_value any more).
# Pass 28: locate the header instead of hard-coding the old path.
toml_h = next((p for p in ("tools/common/toml.h", "tools/minicargo/toml.h")
               if os.path.exists(p)), None)
if toml_h is None:
    print("  !! FILE MISSING (upstream drift?): tools/{common,minicargo}/toml.h"); ok = False
else:
    s = open(toml_h).read()
    if "Pass 28: coerce scalar numerics/bools to text" in s:
        print("  already patched: toml.h as_string coercion")
    else:
        old_as = """    const ::std::string& as_string() const {
        if( m_type != Type::String ) {
            throw TypeError { m_type, Type::String };
        }
        return m_str_value;
    }"""
        new_as = """    const ::std::string& as_string() const {
        // Pass 28: coerce scalar numerics/bools to text (manifest fields such as
        // `version = 1.99.0` arrive as Integer in minicargo's parser).
        if( m_type == Type::String ) { return m_str_value; }
        if( m_type == Type::Integer ) { m_str_value = ::std::to_string(m_int_value); return m_str_value; }
        if( m_type == Type::Boolean ) { m_str_value = (m_int_value != 0 ? "true" : "false"); return m_str_value; }
        throw TypeError { m_type, Type::String };
    }"""
        if old_as not in s:
            print("  !! ANCHOR NOT FOUND (upstream drift?): toml.h as_string"); ok = False
        elif "::std::string   m_str_value;" not in s:
            print("  !! ANCHOR NOT FOUND (upstream drift?): toml.h m_str_value decl"); ok = False
        else:
            s = s.replace(old_as, new_as, 1)
            s = s.replace("::std::string   m_str_value;",
                          "mutable ::std::string   m_str_value;", 1)
            open(toml_h, "w").write(s)
            print(f"  patched: {toml_h} as_string coercion + mutable m_str_value")

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
