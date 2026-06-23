#!/usr/bin/env python
"""Publish GitHub Release v0.2.6 — creates release + uploads assets."""
import subprocess, json, sys, os, urllib.parse, urllib.request

REPO = "andywongpt-my/desktop-usage-helper"
VERSION = "0.2.6"
BUNDLE = r"C:\Users\andyw\projects\desktop-usage-helper\src-tauri\target\release\bundle\nsis"
LATEST_JSON = r"C:\Users\andyw\projects\desktop-usage-helper\latest.json"

# Get token from git credential store
cred_input = b"protocol=https\nhost=github.com\n\n"
result = subprocess.run(["git", "credential", "fill"], input=cred_input, capture_output=True)
lines = result.stdout.decode().strip().split("\n")
token = ""
for line in lines:
    if line.startswith("password="):
        token = line[len("password="):]
        break

if not token:
    print("ERROR: No GitHub token found")
    sys.exit(1)

print(f"Token length: {len(token)}")

# 1. Create release
print(f"=== Creating release v{VERSION} ===")
body_data = json.dumps({
    "tag_name": f"v{VERSION}",
    "name": f"v{VERSION} — Custom endpoint, HTTP 411, updater fix",
    "body": """Desktop Usage Helper v0.2.6

## Fixes

- **Custom API endpoint**: Users can now set a custom endpoint URL for any provider (Ollama, MiniMax, OpenAI, Anthropic, Z.ai) in Settings. Previously the URL was hardcoded with no override option.
- **HTTP 411 "Length Required"**: MiniMax and Ollama POST requests now include an explicit `Content-Length: 0` header. reqwest 0.12 doesn't auto-send this header even with `.body("")`, causing Ollama's Google frontend to reject with HTTP 411.
- **Auto-updater signing key mismatch**: The signing key was regenerated in v0.2.5 but the pubkey in `tauri.conf.json` was never updated, causing updater verification failure on new builds. Generated a new tauri-native key pair and updated the pubkey.
- **CSP fix**: Path-based CSP entries (`https://github.com/andywongpt-my/...`) replaced with origin-based (`https://github.com`). CSP matches by origin, not path.

## Verification
- `cargo check` ✅ (0 errors, 14 pre-existing warnings)
- `npm run build` ✅ (1615 modules)
""",
    "prerelease": False
}).encode()

req = urllib.request.Request(
    f"https://api.github.com/repos/{REPO}/releases",
    data=body_data,
    headers={
        "Authorization": f"token {token}",
        "Content-Type": "application/json",
        "Accept": "application/vnd.github+json"
    },
    method="POST"
)

try:
    with urllib.request.urlopen(req) as resp:
        release = json.loads(resp.read())
    print(f"Release created! ID: {release['id']}")
    print(f"URL: {release['html_url']}")
except urllib.error.HTTPError as e:
    err_body = e.read().decode()
    print(f"HTTP {e.code}: {err_body[:300]}")
    if e.code == 422:
        # Release already exists, get it
        req2 = urllib.request.Request(
            f"https://api.github.com/repos/{REPO}/releases/tags/v{VERSION}",
            headers={"Authorization": f"token {token}", "Accept": "application/vnd.github+json"}
        )
        with urllib.request.urlopen(req2) as resp:
            release = json.loads(resp.read())
        print(f"Using existing release ID: {release['id']}")
    else:
        sys.exit(1)

release_id = release["id"]

# 2. Upload assets
def upload_asset(filepath, name):
    print(f"  Uploading {name}...")
    encoded_name = urllib.parse.quote(name)
    with open(filepath, "rb") as f:
        data = f.read()
    req = urllib.request.Request(
        f"https://uploads.github.com/repos/{REPO}/releases/{release_id}/assets?name={encoded_name}",
        data=data,
        headers={
            "Authorization": f"token {token}",
            "Content-Type": "application/octet-stream",
            "Accept": "application/vnd.github+json"
        },
        method="POST"
    )
    try:
        with urllib.request.urlopen(req) as resp:
            result = json.loads(resp.read())
        print(f"  -> OK ({result.get('size', '?')} bytes)")
    except urllib.error.HTTPError as e:
        err = e.read().decode()
        print(f"  -> HTTP {e.code}: {err[:200]}")

print()
print("=== Uploading assets ===")
upload_asset(os.path.join(BUNDLE, f"Desktop Usage Helper_{VERSION}_x64-setup.exe"), f"Desktop.Usage.Helper_{VERSION}_x64-setup.exe")
upload_asset(os.path.join(BUNDLE, f"Desktop Usage Helper_{VERSION}_x64-setup.exe.sig"), f"Desktop.Usage.Helper_{VERSION}_x64-setup.exe.sig")
upload_asset(LATEST_JSON, "latest.json")

# 3. Set release as latest, demote old release
print()
print("=== Setting as latest ===")
req = urllib.request.Request(
    f"https://api.github.com/repos/{REPO}/releases/{release_id}",
    data=json.dumps({"draft": False, "make_latest": "true"}).encode(),
    headers={
        "Authorization": f"token {token}",
        "Content-Type": "application/json",
        "Accept": "application/vnd.github+json"
    },
    method="PATCH"
)
with urllib.request.urlopen(req) as resp:
    result = json.loads(resp.read())
    print(f"  -> make_latest: {result.get('make_latest')}")

# Demote v0.2.5 release
print("=== Demoting v0.2.5 ===")
req = urllib.request.Request(
    f"https://api.github.com/repos/{REPO}/releases/tags/v0.2.5",
    headers={"Authorization": f"token {token}", "Accept": "application/vnd.github+json"}
)
try:
    with urllib.request.urlopen(req) as resp:
        old_release = json.loads(resp.read())
    old_id = old_release["id"]
    req = urllib.request.Request(
        f"https://api.github.com/repos/{REPO}/releases/{old_id}",
        data=json.dumps({"make_latest": "false"}).encode(),
        headers={
            "Authorization": f"token {token}",
            "Content-Type": "application/json",
            "Accept": "application/vnd.github+json"
        },
        method="PATCH"
    )
    with urllib.request.urlopen(req) as resp:
        print(f"  -> v0.2.5 demoted")
except Exception as e:
    print(f"  -> Could not demote v0.2.5: {e}")

# 4. Push tag
print()
print("=== Pushing tag ===")
os.system(f'git tag v{VERSION} 2>/dev/null || true')
os.system(f'git push origin v{VERSION} 2>/dev/null || true')

# 5. Verify
print()
print("=== Verification ===")
req = urllib.request.Request(
    f"https://api.github.com/repos/{REPO}/releases/{release_id}/assets",
    headers={"Authorization": f"token {token}", "Accept": "application/vnd.github+json"}
)
with urllib.request.urlopen(req) as resp:
    assets = json.loads(resp.read())
for a in assets:
    print(f"  {a['name']} ({a['size']} bytes)")
    print(f"    -> {a['browser_download_url']}")

print()
print("=== DONE ===")
print(f"Release: https://github.com/{REPO}/releases/tag/v{VERSION}")
print(f"latest.json: https://github.com/{REPO}/releases/download/v{VERSION}/latest.json")