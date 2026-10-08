# Standing project authorization

The standalone Rust/Bevy rewrite and its automatic loop remain parked since
5 October 2026; do not resume paused issues without a new owner direction.
On 6 October the owner clarified the full existing-Rust PC mod goal: MW2-inspired
gunplay and skateboarding ([RUST-MW2-SKATE.md](RUST-MW2-SKATE.md), issue #275). Issue #266
covers the loadout/damage component only. Implementation, debugging and verification
are authorized. Use open-source code and original authored content; retain licences.
Keep game/server files outside GitHub, player text English and historical rewrite work.

On 8 October 2026 the owner chose the server-only (Oxide) mod. Client-loader,
generator and interop work is paused; do not resume it without a new owner
direction ([reason and limits](RUST-CLIENT-LOADER.md#owner-direction-8-october-2026-loader-work-paused)).
Visible skateboarding and client recoil/ADS are deferred with it. The owner
creates the DigitalOcean test host himself; agents order nothing.

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
