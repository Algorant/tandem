#!/usr/bin/env python3
"""Focused release-note and publication assertions for `just release`."""

import json
import pathlib
import re
import sys


EXPECTED_ASSETS = {
    "tandem-installer.sh",
    "tandem-x86_64-unknown-linux-gnu.tar.xz",
    "tandem-aarch64-unknown-linux-gnu.tar.xz",
    "tandem-x86_64-apple-darwin.tar.xz",
    "tandem-aarch64-apple-darwin.tar.xz",
    "sha256.sum",
}


def fail(message: str) -> None:
    raise SystemExit(message)


def release_section(version: str) -> str:
    path = pathlib.Path("RELEASES.md")
    if not path.is_file():
        fail("RELEASES.md is required before a release")
    text = path.read_text(encoding="utf-8")
    heading = re.compile(rf"^##[ \t]+{re.escape(version)}[ \t]*$", re.MULTILINE)
    matches = list(heading.finditer(text))
    if len(matches) != 1:
        fail(f"RELEASES.md must contain exactly one '## {version}' section; found {len(matches)}")
    start = matches[0].end()
    next_heading = re.search(r"^##(?:[ \t]|$)", text[start:], re.MULTILINE)
    section = text[start : start + next_heading.start() if next_heading else len(text)].strip()
    meaningful_lines = [
        line.strip()
        for line in section.splitlines()
        if line.strip() and not re.match(r"^#{1,6}(?:[ \t]|$)", line.strip())
    ]
    if not meaningful_lines or not any(any(char.isalnum() for char in line) for line in meaningful_lines):
        fail(f"RELEASES.md section {version} must contain meaningful release notes")
    return section


def check_cargo_version(version: str) -> None:
    cargo = pathlib.Path("tandem/Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'(?m)^version = "([^"]+)"$', cargo)
    if not match or match.group(1) != version:
        fail(f"tandem/Cargo.toml version must agree with requested version {version}")


def check_manifest(notes_path: pathlib.Path, manifest_path: pathlib.Path) -> None:
    notes = notes_path.read_text(encoding="utf-8").strip()
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if notes not in manifest.get("announcement_github_body", ""):
        fail("cargo-dist announcement body does not include the curated RELEASES.md section")


def check_published_release(notes_path: pathlib.Path, release_path: pathlib.Path) -> None:
    notes = notes_path.read_text(encoding="utf-8").strip()
    release = json.loads(release_path.read_text(encoding="utf-8"))
    if release.get("isDraft"):
        fail("GitHub Release is still a draft")
    if release.get("isPrerelease"):
        fail("GitHub Release is unexpectedly marked prerelease")
    if notes not in release.get("body", ""):
        fail("GitHub Release body does not include the curated RELEASES.md section")
    assets = {asset["name"]: asset.get("size", 0) for asset in release.get("assets", [])}
    missing = sorted(EXPECTED_ASSETS - assets.keys())
    if missing:
        fail(f"GitHub Release is missing expected assets: {', '.join(missing)}")
    empty = sorted(name for name in EXPECTED_ASSETS if assets[name] <= 0)
    if empty:
        fail(f"GitHub Release has empty expected assets: {', '.join(empty)}")


def main() -> None:
    if len(sys.argv) < 2:
        fail("usage: release_checks.py {notes VERSION OUTPUT|cargo VERSION|manifest NOTES MANIFEST|published NOTES RELEASE}")
    command, *args = sys.argv[1:]
    if command == "notes" and len(args) == 2:
        pathlib.Path(args[1]).write_text(release_section(args[0]) + "\n", encoding="utf-8")
    elif command == "cargo" and len(args) == 1:
        check_cargo_version(args[0])
    elif command == "manifest" and len(args) == 2:
        check_manifest(pathlib.Path(args[0]), pathlib.Path(args[1]))
    elif command == "published" and len(args) == 2:
        check_published_release(pathlib.Path(args[0]), pathlib.Path(args[1]))
    else:
        fail("usage: release_checks.py {notes VERSION OUTPUT|cargo VERSION|manifest NOTES MANIFEST|published NOTES RELEASE}")


if __name__ == "__main__":
    main()
