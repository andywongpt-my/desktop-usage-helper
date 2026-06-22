#!/usr/bin/env bash
# Publish a GitHub release with auto-updater manifest (latest.json).
#
# Prereqs:
#   1. Build signed installers (pass key CONTENT, not path — Tauri ignores _PATH):
#        TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/desktop-usage-helper.key)" \
#        TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
#        npm run tauri:build
#   2. Set GH_TOKEN for GitHub API:
#        export GH_TOKEN="ghp_xxxx"
#   3. Run this script:
#        scripts/publish-release.sh
#
# Env:
#   GH_TOKEN       — GitHub PAT with repo:write (or use `gh auth login`)
#   VERSION        — override version (default: read from tauri.conf.json)
#   RELEASE_NOTES  — override release body text (default: "Desktop Usage Helper v$VERSION")

set -euo pipefail

REPO="andywongpt-my/desktop-usage-helper"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUNDLE_DIR="$PROJECT_DIR/src-tauri/target/release/bundle"

# Read version from tauri.conf.json
VERSION="${VERSION:-$(grep -o '"version": *"[^"]*"' "$PROJECT_DIR/src-tauri/tauri.conf.json" | head -1 | sed 's/.*"version": *"//;s/"//')}"
if [ -z "$VERSION" ]; then
  echo "ERROR: could not determine version from tauri.conf.json"
  exit 1
fi

RELEASE_NOTES="${RELEASE_NOTES:-Desktop Usage Helper v$VERSION}"

echo "Publishing release v$VERSION for $REPO"

# --- Collect artifacts -------------------------------------------------------
# Tauri v2 with createUpdaterArtifacts produces:
#   nsis/*.exe          — NSIS installer
#   nsis/*.exe.sig      — signature for updater
#   msi/*.msi           — MSI installer
#   msi/*.msi.sig       — signature for updater

NSIS_SETUP="$BUNDLE_DIR/nsis/Desktop Usage Helper_${VERSION}_x64-setup.exe"
NSIS_SIG="$BUNDLE_DIR/nsis/Desktop Usage Helper_${VERSION}_x64-setup.exe.sig"
MSI_FILE="$BUNDLE_DIR/msi/Desktop Usage Helper_${VERSION}_x64_en-US.msi"
MSI_SIG="$BUNDLE_DIR/msi/Desktop Usage Helper_${VERSION}_x64_en-US.msi.sig"

# Verify the signed updater artifacts exist
if [ ! -f "$NSIS_SETUP" ] || [ ! -f "$NSIS_SIG" ]; then
  echo "ERROR: Signed updater artifacts not found."
  echo "  Expected: $NSIS_SETUP"
  echo "  Expected: $NSIS_SIG"
  echo ""
  echo "Build with:"
  echo "  TAURI_SIGNING_PRIVATE_KEY=\"\$(cat ~/.tauri/desktop-usage-helper.key)\" \\"
  echo "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD=\"\" \\"
  echo "  npm run tauri:build"
  exit 1
fi

# URL-encode spaces for GitHub asset filenames (space → %20)
NSIS_SETUP_URLSAFE=$(basename "$NSIS_SETUP" | sed 's/ /%20/g')
NSIS_SIG_URLSAFE=$(basename "$NSIS_SIG" | sed 's/ /%20/g')

SIG_NSIS=$(cat "$NSIS_SIG")

# --- Create GitHub Release ----------------------------------------------------

# Check if release already exists
RELEASE_ID=$(curl -sf -H "Authorization: token ${GH_TOKEN}" \
  "https://api.github.com/repos/$REPO/releases/tags/v$VERSION" 2>/dev/null \
  | grep -o '"id": *[0-9]*' | head -1 | sed 's/.*: *//' || true)

if [ -n "$RELEASE_ID" ]; then
  echo "Release v$VERSION already exists (ID: $RELEASE_ID) — uploading assets only"
else
  echo "Creating new release v$VERSION..."
  RELEASE_ID=$(curl -sf -X POST \
    -H "Authorization: token ${GH_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"tag_name\":\"v$VERSION\",\"name\":\"v$VERSION — Desktop Usage Helper\",\"body\":\"${RELEASE_NOTES}\",\"prerelease\":false}" \
    "https://api.github.com/repos/$REPO/releases" \
    | grep -o '"id": *[0-9]*' | head -1 | sed 's/.*: *//')

  if [ -z "$RELEASE_ID" ]; then
    echo "ERROR: failed to create release"
    exit 1
  fi
  echo "Created release ID: $RELEASE_ID"
fi

# --- Upload assets ------------------------------------------------------------

upload_asset() {
  local file="$1"
  local label=$(basename "$file")
  echo "  Uploading $label..."

  # Check if asset already exists, delete if so
  EXISTING=$(curl -sf -H "Authorization: token ${GH_TOKEN}" \
    "https://api.github.com/repos/$REPO/releases/$RELEASE_ID/assets" \
    | grep -o "\"name\": *\"$label\"" || true)

  if [ -n "$EXISTING" ]; then
    ASSET_ID=$(curl -sf -H "Authorization: token ${GH_TOKEN}" \
      "https://api.github.com/repos/$REPO/releases/$RELEASE_ID/assets" \
      | grep -A2 "\"name\": *\"$label\"" | grep -o '"id": *[0-9]*' | head -1 | sed 's/.*: *//')
    if [ -n "$ASSET_ID" ]; then
      curl -sf -X DELETE -H "Authorization: token ${GH_TOKEN}" \
        "https://api.github.com/repos/$REPO/releases/assets/$ASSET_ID" || true
    fi
  fi

  curl -sf -X POST \
    -H "Authorization: token ${GH_TOKEN}" \
    -H "Content-Type: application/octet-stream" \
    --data-binary @"$file" \
    "https://uploads.github.com/repos/$REPO/releases/$RELEASE_ID/assets?name=$label" > /dev/null
  echo "  ✓ $label uploaded"
}

upload_asset "$NSIS_SETUP"
upload_asset "$NSIS_SIG"
[ -f "$MSI_FILE" ] && upload_asset "$MSI_FILE"
[ -f "$MSI_SIG" ] && upload_asset "$MSI_SIG"

# --- Generate latest.json manifest -------------------------------------------

PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$BUNDLE_DIR/latest.json" << EOF
{
  "version": "$VERSION",
  "notes": "$RELEASE_NOTES",
  "pub_date": "$PUB_DATE",
  "platforms": {
    "windows-x86_64": {
      "signature": "$SIG_NSIS",
      "url": "https://github.com/$REPO/releases/download/v$VERSION/${NSIS_SETUP_URLSAFE}"
    }
  }
}
EOF

echo ""
echo "Generated latest.json:"
cat "$BUNDLE_DIR/latest.json"
echo ""

upload_asset "$BUNDLE_DIR/latest.json"

echo ""
echo "✅ Release v$VERSION published with auto-updater manifest."
echo "   latest.json is available at:"
echo "   https://github.com/$REPO/releases/download/v$VERSION/latest.json"
