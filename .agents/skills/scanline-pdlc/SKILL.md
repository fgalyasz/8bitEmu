---
name: scanline-pdlc
description: >-
  Run the Scanline product PDLC for a new feature or user-visible bugfix.
  Use when the user wants a feature, a product bugfix, PRD, epic, story, release,
  or says PDLC / tervezés / építsük meg.
---

# Scanline PDLC

Read and follow `{project-root}/docs/pdlc.md`. Do not skip steps on a Feature or Fix track.

## On activation

1. Classify Feature / Fix / Chore.
2. If Chore, implement the change only.
3. If Feature or Fix, execute the pipeline in `docs/pdlc.md` in order.
4. Scaffold with `{project-root}/scripts/pdlc-new.sh <slug>` when starting a new PRD folder.
5. Create GitHub issues with `gh issue create` (`--parent` for stories and sub-issues). Add each issue to project **#10** (`fgalyasz`, Scanline) with `{project-root}/scripts/pdlc-project-item.sh <n> "Todo"`. Move Status through In Progress → Done. Templates live in `.github/ISSUE_TEMPLATE/`.
6. After `cargo test` is green, review the diff against FR consequences.
7. Update `CHANGELOG.md`. Skip DMG and website until a public increment exists.
8. Commit and `git push origin HEAD`. Close shipped issues and set Project Status to `Done`.

Speak Hungarian to the user. Write artifacts in English.
