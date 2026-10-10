# Rust skate client source review

Codex reviewed [client PR #340](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/340) for [#337 item 12](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337), 10 October 2026.
Reviewed head: `8f5ce049f36b8789d06a7a94291d92948df3e6eb`.
The [inline review](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/340#pullrequestreview-5478070624)
contains exact source anchors and correction requests. Findings apply to this pin.

## State and render corrections (P2)

1. **Reset on loss of either cached identity.** SkateRig's reset requires a surviving
   Local and missing Body. If both disappear before Tick, rediscovery preserves old
   AwakeAt/lateAdded/RiderRig.Bound and never creates the required fresh late driver
   after the replacement walk component. Reset independently, then rebind/recreate.
2. **Suspend banking throughout physical air time.** Unjumped takeoff and normal
   grind exit set Mode=Air but TrickAir=false; AirStep submits airborne=false.
   An open combo banks after 1.5 s during a long drop, before a later impact bail
   can discard it. Separate physical air/combo suspension from trick eligibility.
3. **Carry rising velocity through a near-support mount.** Mount enters Ground;
   at clearance .1 m, FeetProbe=.3 m still reports support and GroundStep replaces
   incoming +6 m/s ascent with −.8 m/s. Preserve ascent/pending takeoff on mount.
4. **Refresh renderer identities after clothing changes.** RiderRig refreshes only
   when total mesh count changes. Same-count replacement leaves stale body/leg arrays
   and third=true; new meshes miss visibility changes even when camera mode toggles.
5. **Allow recovery from late-driver creation failure.** lateAdded becomes true
   before AddComponent succeeds. Throw/null permanently skips retry while CanMount
   requires a driver. Mark success after a non-null result; retain recovery state.

## Failure/input follow-ups (P3)

- Stop/silence owned audio on playback failure. The failed flag prevents further
  updates, so an already audible persistent loop cannot fade out on dismount.
- Clear latched physics presses when cursor/keyboard input is blocked. Clearing
  held keys alone leaves a queued jump/flip/spin for the next physics step.

## Outcome

Claude changed the client for all seven findings in `01e8d5e` and answered each in the inline
review. Each has an offline check against a stand-in engine outside this repository (reconnect,
fall-combo, mount-rising, clothing, driver-retry, sound-failure, seat); none of these paths was
reproduced or rerun in the game. Later client commits are not covered by this review.

## Evidence and limits

Independent source/caller reviews confirmed the five P2 paths; root checked the
combined transitions and two follow-ups. Normal grind continuation/release cooldown
and landed switch/nose mapping had no additional concrete blocker in this review.
SHA-256 of the reviewed main files:

| File | SHA-256 |
| --- | --- |
| SkateRide.cs | `8973b8d84d708cdd5dfac1491a3731fb1e3e949c69b929fdd6bfb9bbbd5d0f02` |
| SkateGrind.cs | `76d12d1ea98de1f1999aceb4bb5abcb4ea48667981c4e235bb938277a4891c09` |
| RiderRig.cs | `d7f0441896127f832db4805516452e7bd52a8025eeb2224da004a8f0b927bfd3` |

No client edits/build/execution, Shadow PC or server use. Native occurrence of these
source scenarios, lifecycle/feel/audio/grind and second-client acceptance were not
verified. See [reported native facts](RUST-SKATE-CLIENT.md) and [remaining work](../TODO.md).
