#!/bin/bash
# NROS snapshot dance (repo convention; tricks #14/#15).
#
# Run AFTER making a content commit you intend to push. It re-pins both snapshot
# manifests to that content commit (HEAD) and refreshes the git-blob-SHA-1 fingerprints
# of every pinned manifest, then creates the second (re-pin) commit. The representation
# gate resolves blobs at snapshot.source_revision — with the dance, that revision is
# always the pushed content commit. CI checkouts therefore need fetch-depth: 0 (F-20).
#
# Usage:  bash snapshot-dance.sh
# (run in the repo root with a clean tree except for the two snapshot files, which this
#  script overwrites from git anyway)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
SRC=$(git rev-parse HEAD)   # content commit
python3 - "$SRC" <<'PY'
import re, subprocess, sys
src = sys.argv[1]
def blob(path):
    return subprocess.run(["git","rev-parse",f"{src}:{path}"],
                          capture_output=True,text=True,check=True).stdout.strip()

# --- docs/representation/snapshot.yaml ---
p = "docs/representation/snapshot.yaml"
s = open(p).read()
s = re.sub(r'(source_revision:\n  commit: ")[0-9a-f]{40}(")', r"\g<1>"+src+r"\g<2>", s)
for m in ["architecture.yaml","capabilities.yaml","evidence.yaml","claims.yaml"]:
    h = blob("docs/representation/"+m)
    s, n = re.subn(r"(    "+m+r': ")[0-9a-f]{40}(")', r"\g<1>"+h+r"\g<2>", s)
    assert n == 1, m
open(p,"w").write(s)

# --- docs/documentation/snapshot.yaml ---
p2 = "docs/documentation/snapshot.yaml"
s2 = open(p2).read()
s2 = re.sub(r'(source_revision:\n  commit: )[0-9a-f]{40}', r"\g<1>"+src, s2)
for doc in ["schema.yaml","inventory.yaml","authorities.yaml","relationships.yaml","references.yaml"]:
    h = blob("docs/documentation/"+doc)
    pat = re.compile(r"(- id: DOC-[A-Z-]+\n    path: docs/documentation/"+doc+r"\n    blob_sha: )[0-9a-f]{40}")
    s2, n = pat.subn(r"\g<1>"+h, s2)
    assert n == 1, "blob line for "+doc+" not found"
open(p2,"w").write(s2)
print("re-pinned to", src)
PY
git add docs/representation/snapshot.yaml docs/documentation/snapshot.yaml
git -c user.name="${GIT_AUTHOR_NAME:-Arena Agent}" -c user.email="${GIT_AUTHOR_EMAIL:-arena@arena.ai}" \
  commit -m "Re-pin snapshots to content commit $SRC (dance 2nd commit)" --quiet
echo DANCE_DONE
