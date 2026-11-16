#!/usr/bin/env bash
# Remove preinstalled toolchains PRL never uses, so Rust/Bevy builds have room.
# Safe to re-run: missing paths are skipped. Keeps Rust, Python, Node, git,
# Chrome (Playwright) and the Linux build/Xvfb libraries.
set -euo pipefail

SUDO=""
if [ "$(id -u)" -ne 0 ] && command -v sudo >/dev/null 2>&1; then
  SUDO="sudo"
fi

UNUSED=(
  /opt/hostedtoolcache/CodeQL
  /usr/share/dotnet
  /usr/local/lib/android
  /opt/ghc
  /usr/local/.ghcup
  /usr/share/swift
  /usr/local/julia*
  /usr/share/miniconda
  /usr/lib/jvm
  /opt/az
  /opt/microsoft/powershell
  /usr/lib/google-cloud-sdk
)

echo "Before: $(df -h / | awk 'NR==2 {print $4}') free"
for path in "${UNUSED[@]}"; do
  for match in $path; do
    if [ -e "$match" ]; then
      echo "Removing $match ($(du -sh "$match" 2>/dev/null | cut -f1))"
      $SUDO rm -rf "$match"
    fi
  done
done
echo "After: $(df -h / | awk 'NR==2 {print $4}') free"
