"""Print archive/package/url/sha1 of the current CEF build for GITHUB_OUTPUT."""

import json
import os
import sys
import urllib.parse
import urllib.request
from datetime import datetime, timedelta, timezone

platform = os.environ["CEF_PLATFORM"]
channel = os.environ["CEF_CHANNEL"]
file_type = os.environ["CEF_FILE_TYPE"]

def branch(build):
    return int(build["chromium_version"].split(".")[2])

def is_beta(build):
    return "_beta" in build["files"][0]["name"]

def current_stable(versions):
    depth = 5
    selected = None
    selected_branch = 0
    for build in versions:
        if is_beta(build):
            continue
        build_branch = branch(build)
        if build_branch > selected_branch:
            selected = build
            selected_branch = build_branch
        depth -= 1
        if depth == 0:
            break
    return selected

def current_beta(versions, stable_build):
    stable_branch = branch(stable_build) if stable_build is not None else 0
    max_age = timedelta(days=60)
    now = datetime.now(timezone.utc)
    for build in versions:
        if not is_beta(build):
            continue
        timestamp = datetime.fromisoformat(
            build["files"][0]["last_modified"].replace("Z", "+00:00")
        )
        if branch(build) > stable_branch and now - timestamp < max_age:
            return build
        break
    return None

with urllib.request.urlopen("https://cef-builds.spotifycdn.com/index.json", timeout=60) as response:
    builds = json.load(response)

versions = builds[platform]["versions"]
stable_build = current_stable(versions)
if channel == "stable":
    selected_build = stable_build
elif channel == "beta":
    selected_build = current_beta(versions, stable_build)
else:
    sys.exit(f"Unsupported CEF channel: {channel}")

selected_file = None
if selected_build is not None:
    selected_file = next(
        (
            file
            for file in selected_build["files"]
            if file["type"] == file_type and file["name"].endswith(".tar.bz2")
        ),
        None,
    )

if selected_build is None or selected_file is None:
    sys.exit(f"No {platform} {channel} {file_type} CEF binary found")

archive = selected_file["name"]
package = archive.removesuffix(".tar.bz2")
url = "https://cef-builds.spotifycdn.com/" + urllib.parse.quote(archive)

print(f"archive={archive}")
print(f"package={package}")
print(f"url={url}")
print(f"sha1={selected_file['sha1']}")
print(f"cef_version={selected_build['cef_version']}")
print(f"chromium_version={selected_build['chromium_version']}")
