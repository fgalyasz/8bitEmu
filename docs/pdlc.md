# Scanline product PDLC

Every **new product feature** and every **user-visible bugfix** follows this path. The agent does not wait for the user to list the steps. Chat stays Hungarian; artifacts, code, GitHub, and commit messages stay English.

## Track

| Track | When | Skip |
| ----- | ---- | ---- |
| **Feature** | New capability or user-visible change | Nothing below |
| **Fix** | Bug with a product consequence | Coaching-path PRD; still write a thin PRD or investigation + FRs |
| **Chore** | Typo, refactor with no behavior change, process-only | PRD, GitHub epic/project items, DMG, website copy |

## Pipeline

1. **Classify** — Feature, fix, or chore. If chore, do the change and stop.
2. **BMad planning** — Fast path unless Ferenc asks for coaching. Record decisions in `.decision-log.md`.
3. **PRD** — `_bmad-output/planning-artifacts/prds/prd-Scanline-<slug>/` with `prd.md`, `.decision-log.md`, `addendum.md`, `epics.md`. Hobby/solo, ~2 pages. Stable FR IDs. `./scripts/pdlc-new.sh <slug>` scaffolds the folder.
4. **Epic + stories + GitHub Project** — `gh issue create` `[EPIC] …` then `[STORY]` / `[DEV]` / `[TEST]` sub-issues with `--parent`. Link the PRD in the epic. Put `parent_issue` in PRD frontmatter. Add **every** epic, story, and sub-issue to [Scanline project #10](https://github.com/users/fgalyasz/projects/10) and set Status:
   - created → `Todo`
   - implementation started → `In Progress`
   - shipped and issue closed → `Done`
   Helper: `./scripts/pdlc-project-item.sh <n> "Todo"`.
5. **Dev** — English names. Core logic stays in `scanline-core`. The window does not bilinear-filter the frame.
6. **Unit tests** — New core logic ≥95% line coverage. Positive and negative. No timing-sensitive asserts. `cargo test` must pass.
7. **Review** — Diff vs FR consequences. If the diff is large, security-sensitive, or Ferenc asks, run `bmad-code-review`. Fix blockers before build.
8. **Changelog + website** — `CHANGELOG.md` first. User-facing copy when the site must explain the change. DMG and tenprintsoftware.com wait until a public increment.
9. **Commit + push** — One release commit after tests. Push `origin HEAD`. Close the GitHub stories and epic. Set those Project items to `Done`.

## Do not

- Skip the PRD because the change “is obvious”.
- Push without green `cargo test`.
- Commit secrets.
- Force-push `main`.
- Bundle Spectrum or C64 ROMs.
- Ship a neural upscaler.

## References

- Issue templates: `.github/ISSUE_TEMPLATE/`
- PRD stubs: `docs/pdlc-templates/`
- GitHub Project: [users/fgalyasz/projects/10](https://github.com/users/fgalyasz/projects/10)

If `gh project` returns Forbidden, refresh token scopes: `gh auth refresh -s project`.
