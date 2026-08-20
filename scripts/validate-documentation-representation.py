#!/usr/bin/env python3
"""Validate the NROS documentation representation manifests."""

from __future__ import annotations

import hashlib
import pathlib
import sys
from typing import Any

try:
    import yaml
except ImportError as exc:  # pragma: no cover
    raise SystemExit("PyYAML is required: python -m pip install pyyaml") from exc

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs" / "documentation"
MANIFESTS = {
    "schema": DOCS / "schema.yaml",
    "inventory": DOCS / "inventory.yaml",
    "authorities": DOCS / "authorities.yaml",
    "relationships": DOCS / "relationships.yaml",
    "references": DOCS / "references.yaml",
    "snapshot": DOCS / "snapshot.yaml",
}


def load(path: pathlib.Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        value = yaml.safe_load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"{path}: top-level YAML value must be a mapping")
    return value


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    errors: list[str] = []
    data: dict[str, dict[str, Any]] = {}

    for name, path in MANIFESTS.items():
        if not path.is_file():
            errors.append(f"missing manifest: {path.relative_to(ROOT)}")
            continue
        try:
            data[name] = load(path)
        except (OSError, ValueError, yaml.YAMLError) as exc:
            errors.append(str(exc))

    if errors:
        return report(errors)

    schema = data["schema"]
    inventory = data["inventory"]
    authorities = data["authorities"]
    relationships = data["relationships"]
    references = data["references"]
    snapshot = data["snapshot"]

    if schema.get("schema_version") != inventory.get("schema_version"):
        errors.append("schema_version mismatch between schema and inventory")

    docs = inventory.get("documents", [])
    ids = [item.get("id") for item in docs if isinstance(item, dict)]
    paths = [item.get("path") for item in docs if isinstance(item, dict)]

    if len(ids) != len(set(ids)):
        errors.append("inventory contains duplicate document IDs")
    if len(paths) != len(set(paths)):
        errors.append("inventory contains duplicate document paths")

    document_ids = set(ids)
    for item in docs:
        if not isinstance(item, dict):
            errors.append("inventory contains a non-mapping document record")
            continue
        for field in ("id", "path", "type", "authority", "status", "purpose"):
            if not item.get(field):
                errors.append(f"inventory document missing required field: {field}")
        path = ROOT / str(item.get("path", ""))
        if not path.is_file():
            errors.append(f"inventory path does not exist: {item.get('path')}")

    allowed_relations = set(schema.get("vocabularies", {}).get("relationship_types", []))
    for item in relationships.get("relationships", []):
        if not isinstance(item, dict):
            errors.append("relationship is not a mapping")
            continue
        source = item.get("source")
        target = item.get("target")
        relation = item.get("relation")
        if source not in document_ids:
            errors.append(f"relationship source missing from inventory: {source}")
        if target not in document_ids:
            errors.append(f"relationship target missing from inventory: {target}")
        if relation not in allowed_relations:
            errors.append(f"unknown relationship type: {relation}")

    for item in references.get("references", []):
        if not isinstance(item, dict):
            errors.append("reference is not a mapping")
            continue
        source = item.get("source")
        target = item.get("target")
        status = item.get("status")
        if source not in document_ids:
            errors.append(f"reference source missing from inventory: {source}")
        if target.startswith("DOC-") and target not in document_ids:
            errors.append(f"internal reference target missing from inventory: {target}")
        if status not in set(schema.get("vocabularies", {}).get("reference_status", [])):
            errors.append(f"unknown reference status: {status}")

    snapshot_docs = {item.get("id"): item for item in snapshot.get("documents", []) if isinstance(item, dict)}
    for document_id, item in snapshot_docs.items():
        path = ROOT / str(item.get("path", ""))
        if document_id not in document_ids:
            errors.append(f"snapshot document missing from inventory: {document_id}")
        if not path.is_file():
            errors.append(f"snapshot path does not exist: {item.get('path')}")
        elif item.get("blob_sha") and len(str(item["blob_sha"])) != 40:
            errors.append(f"snapshot blob SHA is not a Git SHA-1: {document_id}")

    if not snapshot.get("source_revision", {}).get("commit"):
        errors.append("snapshot has no source_revision.commit")

    return report(errors)


def report(errors: list[str]) -> int:
    if errors:
        print("DOCUMENTATION REPRESENTATION: FAIL")
        for error in errors:
            print(f"- {error}")
        return 1
    print("DOCUMENTATION REPRESENTATION: PASS")
    print("- manifests: schema, inventory, authorities, relationships, references, snapshot")
    print("- structural endpoints: valid")
    print("- repository paths: valid")
    print("- reference/relationship vocabulary: valid")
    print("- snapshot identity structure: valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
