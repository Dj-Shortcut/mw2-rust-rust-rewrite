# Standing project authorization

On 5 October 2026 the owner parked the standalone Rust/Bevy rewrite and its
automatic loop. Active scope is the small existing-Rust PC server mod in
[RUST-SERVER-MOD.md](RUST-SERVER-MOD.md) / issue #266; do not resume paused rewrite
issues without a new owner direction. The owner authorizes its implementation,
debugging and verification. Use open-source code and original authored content;
retain licences. Keep installed game/server files outside GitHub. All
player-facing text must remain English. Preserve the historical rewrite work.

Continue from `TODO.md` and open tasks without repeating content confirmations.
The authorization includes project GitHub issues and coordination comments,
isolated feature branches, source commits, pushes and focused PRs. Integrate
routine changes after appropriate verification and review, respecting each open
task's ownership and merge instructions. Development source may be published
before the full product is finished; describe its actual state.

Claim the next task in its GitHub issue before editing, reporting the branch and
owned files. Check existing issues and PRs to avoid duplicate work. Use isolated
worktrees or checkouts; never push to another agent's branch. Keep unrelated work
in separate PRs; a focused correction may use a clearly identified stacked PR.
Task ownership lives in each open issue and its PR, not in this document.
Preserve shared public APIs and agree on any ownership change in the issue.
Issue creation alone does not establish another agent's acknowledgement or work.

The owner prefers parallel subagent work when tasks can be handled independently.
Give each implementation agent distinct files and an isolated worktree; appoint
one integrator for shared files. Preparation and review can run alongside coding.
Use one build owner per shared Cargo target directory and freeze the integrated
source during verification. Report actual results before committing or merging.

Verify changes yourself; the owner does not test intermediate builds. Select
checks that exercise the actual changed behavior, then record evidence and
remaining limits. Temporary probes and logs belong in ignored `context/` and need
no separate permission per check. The existing named-owner-approval policy for
new permanent tests in `crates/approved_tests` remains unchanged.

Keep `TODO.md` honest about code present, headless verification, graphical
verification and release readiness. A compiler check or software-GPU run proves
only its stated scope. Deliver a release only after the agreed product criteria
and complete user flows are verified, with required assets, packaging and start
instructions. An unfinished development build is not a finished product.

The owner's "Full access" setting authorizes work within this scope; host sandbox
rules, available tools and active-session limits still apply. If an actual
platform permission blocks an action, report the action and the stated reason,
and use the platform's permission mechanism when needed. This document does not
keep agents running after sessions stop or grant access to an unavailable app.
Resume using the latest issues, PRs, `TODO.md` and repository instructions.
