# Project Instructions

## Git Workflow

- All changes must be made via Pull Requests (PRs)
- Do not commit directly to the main branch
- Create a feature branch for each change, then open a PR
- Each independent piece of work gets its own feature branch and PR. Do not bundle unrelated changes into a single branch. If there are 5 independent improvements to make, that means 5 branches and 5 PRs.
- Before merging a PR, always check CI status with `gh pr checks <number>`. Do not attempt to merge until all required checks have passed. If checks are failing, investigate and resolve the failures first.
