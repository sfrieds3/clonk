# OpenSpec workflow

This repository's `openspec/` is authoritative. The previous external store is only a backup, not an active source; work from the repository root without a `--store` flag. Existing proposal artifacts are planning, not approval to implement. Review decisions and approval gates before applying changes. CI installs the pinned CLI version in `.github/workflows/ci.yml` with Node 24; use a compatible Node version (at least 22.12) locally.

## References and squash merges

Add the appropriate trailers to every commit, for example `git commit -m 'feat: add colony runtime' --trailer 'OpenSpec-Change: add-desired-state-colony-runtime'` (use `none` for unrelated work). Put a true Git trailer block at the **end of the pull request body**. CI parses the proposed squash message formed from PR title, a blank line, and PR body with `git interpret-trailers --parse`. Each value is one lowercase hyphenated slug naming an active or archived change. Use repeated lines for multiple changes, or `none` alone for unrelated work:

```text
OpenSpec-Change: select-and-migrate-3d-engine
OpenSpec-Change: add-desired-state-colony-runtime
```

```text
OpenSpec-Change: none
```

These are references, not claims that the change is complete. CI checks the PR title/body rather than individual commits. Squash-merge the PR and preserve its trailers in the **actual** squash message; the merge dialog can override the checked text, so the person merging must verify it. If using a merge commit or rebase instead, put the relevant trailers on commits that actually land on main; PR body text alone is not preserved by those methods. Merge commits without trailers are ignored, while referenced inner commits are scanned. Direct pushes bypass PR checks and must carry their own trailers. The only missing-trailer PR exemption is the same-repository `release-please--branches--main` PR authored by `github-actions[bot]`, matching the current release workflow; other bots are not exempt. Archive PRs explicitly use `none`.

## Archive automation

After the main-push Rust check, cross-platform builds, and OpenSpec checks succeed, CI scans every commit in the push range and historical main commits for trailers. Historical scanning recovers references from failed or superseded runs; retain archived change directories so old references remain resolvable. Unknown slugs fail the archive job. A referenced active change is skipped until its planning artifacts are done, its apply state is not blocked, and all of its nonzero tasks are complete. Planning `isComplete` alone is not implementation completion. Ready changes must pass strict validation and the noninteractive OpenSpec archive command, without skipping spec updates or validation. The command uses `--yes` because OpenSpec 1.16 requires confirmation even for ordinary spec updates; independent task and artifact checks precede it, and reported archive warnings fail the job. A failure prevents publication of the entire batch. Already archived changes are idempotent. An automation PR stages only `openspec/**`; review its spec changes and merge it normally. The job skips stale pushes, and later successful pushes rescan history. If the latest push fails CI, fix it and rerun CI or push a fix; no archive PR is created from that failed revision.

GitHub repository Actions settings must allow workflows to create pull requests and the token needs contents and pull-request write permissions. By default the workflow uses `GITHUB_TOKEN`; PRs created with that token generally do **not** trigger other workflows, so required CI must be run by an authorized alternative or the optional `OPENSPEC_ARCHIVE_TOKEN` secret must contain an appropriately scoped GitHub App installation token or PAT that can trigger CI. The generated commit records its source revision in an `OpenSpec-Archive-Base` trailer. PR validation checks the immutable commit marker, its parent and current `origin/main`; if stale, regenerate the archive PR rather than rebasing it. The workflow rechecks main immediately before publication, but main can still advance during or after publication or after a successful check. Configure branch protection to require the OpenSpec check and require branches to be up to date with main before merging; CI alone cannot guarantee freshness at merge time. Configure squash-message conventions separately; this workflow cannot enforce the merge-dialog message or repository settings. No settings are changed automatically.

## Local checks

```sh
python3 -m unittest discover -s scripts/tests
openspec list --json
openspec validate --all --strict --no-interactive
openspec validate --archived --strict --no-interactive
openspec instructions apply --change select-and-migrate-3d-engine --json
```

`python3 scripts/openspec_ci.py check-pr EVENT_JSON` checks an event payload without writing changes. `archive-push BEFORE AFTER` is a mutating CI operation: use only in a disposable clean checkout, never in a working tree with personal edits. It may update main specs and move completed changes even if a later archive fails; the workflow publishes nothing on failure. Neither of the initial active changes should be archived while its implementation tasks remain incomplete.
