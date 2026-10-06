#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $(basename "$0") <slug>" >&2
  echo "Example: $(basename "$0") present-pipe" >&2
  exit 1
fi

slug="$1"
if [[ ! "${slug}" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
  echo "ERROR: slug must be lowercase hyphenated, got: ${slug}" >&2
  exit 1
fi

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dest="${project_root}/_bmad-output/planning-artifacts/prds/prd-8bitEmu-${slug}"
templates="${project_root}/docs/pdlc-templates"

if [[ -d "${dest}" ]]; then
  echo "ERROR: already exists: ${dest}" >&2
  exit 1
fi

mkdir -p "${dest}"
date_today="$(date +%F)"
title="$(echo "${slug}" | tr '-' ' ')"

copy_template() {
  local src_name="$1"
  local dest_name="$2"
  sed -e "s/YYYY-MM-DD/${date_today}/g" \
      -e "s/SLUG/${slug}/g" \
      -e "s/TITLE/${title}/g" \
      "${templates}/${src_name}" > "${dest}/${dest_name}"
}

copy_template prd.md prd.md
copy_template decision-log.md .decision-log.md
copy_template addendum.md addendum.md
copy_template epics.md epics.md

echo "Created ${dest}"
