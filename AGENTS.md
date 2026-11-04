# Agent guidance

The repository-local `openspec/` directory is the authoritative planning root; do not use an external store for this project. Follow the workflow in `openspec/README.md`. Planning is not implementation approval; preserve explicit decision gates and do not archive incomplete work.

Every commit must carry an `OpenSpec-Change: <slug>` trailer for each related change, or `OpenSpec-Change: none` for unrelated work. For example: `git commit -m 'feat: add colony runtime' --trailer 'OpenSpec-Change: add-desired-state-colony-runtime'`. Every pull request must also end its title/body squash message with those trailers, and they must be preserved in the final squash commit message. CI checks the proposed PR squash footer, not every commit. References can point to ongoing changes and are not archive authorization. Automation proposes archives only after implementation tasks and validation are complete; review the generated PR.
