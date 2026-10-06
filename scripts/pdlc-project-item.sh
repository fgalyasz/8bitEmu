#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $(basename "$0") <issue-number-or-url> <status>" >&2
  echo "Status: Todo | In Progress | Done" >&2
  exit 1
}

if [[ $# -lt 2 ]]; then
  usage
fi

raw="$1"
status="$2"
owner="@me"
project_number="10"
repo="fgalyasz/8bitEmu"

case "${status}" in
  Todo|"In Progress"|Done) ;;
  *)
    echo "ERROR: unknown status: ${status}" >&2
    usage
    ;;
esac

if [[ "${raw}" =~ ^https:// ]]; then
  url="${raw}"
else
  url="https://github.com/${repo}/issues/${raw}"
fi

gh project item-add "${project_number}" --owner "${owner}" --url "${url}" >/dev/null
gh project item-edit "${project_number}" --owner "${owner}" --url "${url}" --field Status --value "${status}" >/dev/null
echo "Project ${project_number}: ${url} -> ${status}"
