#!/usr/bin/env python
"""Publish GitHub Release v0.2.1 — creates release + uploads assets."""
import subprocess, json, sys, os, urllib.parse, urllib.request

REPO = "andywongpt-my/desktop-usage-helper"
VERSION = "0.2.1"
BUNDLE = r"C:\Users\andyw\desktop-usage-helper\src-tauri\target\release\bundle\nsis"

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
    "name": f"v{VERSION} — Auto-updater + 5 new providers",
    "body": "Desktop Usage Helper v0.2.1\n\n## What's new\n\n- **Auto-updater**: Settings → Check for Updates (tauri-plugin-updater + tauri-plugin-process)\n- **5 new providers**: Anthropic, OpenAI, Z.ai, Cursor, GitHub Copilot\n- **Hide unused providers** from dashboard\n- **Fix**: API key input losing value on paste/type\n- **Build speed**: rust-lld linker + opt-level s (3min builds)",
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
# Use dot-name convention (matches v0.2.0 release + latest.json URL)
upload_asset(os.path.join(BUNDLE, "Desktop Usage Helper_0.2.1_x64-setup.exe"), "Desktop.Usage.Helper_0.2.1_x64-setup.exe")
upload_asset(os.path.join(BUNDLE, "Desktop Usage Helper_0.2.1_x64-setup.exe.sig"), "Desktop.Usage.Helper_0.2.1_x64-setup.exe.sig")
upload_asset(os.path.join(BUNDLE, "latest.json"), "latest.json")

# 3. Verify
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
