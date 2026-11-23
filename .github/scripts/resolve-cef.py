"""Print archive/package/url/sha1 of the pinned CEF build for GITHUB_OUTPUT.

The bindings must come from the CEF version Spotify ships (`.cef-version`):
struct layouts differ between CEF versions.
"""

import json
import os
import sys
import urllib.parse
import urllib.request

platform = os.environ.get("CEF_PLATFORM", "linux64")
file_type = os.environ.get("CEF_FILE_TYPE", "minimal")
with open(os.environ.get("CEF_VERSION_FILE", ".cef-version"), encoding="utf-8") as f:
    wanted = f.read().strip()

with urllib.request.urlopen("https://cef-builds.spotifycdn.com/index.json", timeout=60) as response:
    builds = json.load(response)

build = next((b for b in builds[platform]["versions"] if b["cef_version"] == wanted), None)
if build is None:
    sys.exit(f"CEF {wanted} is not published for {platform}")
archive = next(
    (f for f in build["files"] if f["type"] == file_type and f["name"].endswith(".tar.bz2")),
    None,
)
if archive is None:
    sys.exit(f"CEF {wanted} has no {file_type} archive for {platform}")

name = archive["name"]
print(f"archive={name}")
print(f"package={name.removesuffix('.tar.bz2')}")
print(f"url=https://cef-builds.spotifycdn.com/{urllib.parse.quote(name)}")
print(f"sha1={archive['sha1']}")
print(f"cef_version={build['cef_version']}")
