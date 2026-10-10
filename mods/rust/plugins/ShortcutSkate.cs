using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Globalization;
using System.Text;
using System.Linq;
using HarmonyLib;
using Newtonsoft.Json;
using Oxide.Core.Plugins;
using Rust;
using Shortcut.RustMod;
using Unity.Collections;
using UnityEngine;

namespace Shortcut.RustMod
{
    public enum SkateSessionReason
    {
        None, Inactive, InvalidClock, BackwardsClock, BatchGap, InvalidDelta,
        InvalidHorizontalPath, InvalidVerticalPath, InvalidMaximumSpeed, InvalidMaximumAirtime,
        InvalidRearmSeconds, HorizontalSpeed, VerticalLimit, Duration, Airtime, Unsupported,
        ClockBeforeDelta, NonIncreasingClock, InvalidTickCount, InvalidCoordinate,
        PositionDiscontinuity, CacheIndexMismatch, PermissionDenied, OptedOut, AutomaticDisabled,
        Unavailable, Disconnected, Npc, Dead, Sleeping, Wounded, Mounted, Swimming, Parented,
        IdentityChanged, ManualOff, ManualOn, AutomaticRearm, ManualRestart, Unload,
        BatchException, MovePatchException, SpeedPatchException, FlyPatchException,
        BatchExhausted, BeginRejected, Count
    }

    public struct SkateSessionObservation
    {
        public SkateSessionReason Reason;
        public double Time, DeltaSeconds, GapSeconds, HorizontalPath, VerticalPath;
        public double MaximumSpeed, MaximumAirtime, AirSeconds, DurationSeconds, ElapsedSeconds;
        public double RearmSeconds, RequiredRearmSeconds, PositionGap;
        public bool FullySupported, SupportKnown, WasActive;
        public long Batch;
        public int SegmentCount, BufferSize;

        public static SkateSessionObservation Empty(double now, SkateSessionReason reason)
        {
            return new SkateSessionObservation { Reason = reason, Time = now,
                DeltaSeconds = double.NaN, GapSeconds = double.NaN,
                HorizontalPath = double.NaN, VerticalPath = double.NaN,
                MaximumSpeed = double.NaN, MaximumAirtime = double.NaN,
                AirSeconds = double.NaN, DurationSeconds = double.NaN, ElapsedSeconds = double.NaN,
                RearmSeconds = double.NaN, RequiredRearmSeconds = double.NaN, PositionGap = double.NaN,
                SegmentCount = -1, BufferSize = -1 };
        }
    }

    public sealed class SkateSessionHistory
    {
        private readonly long[] disarms = new long[(int)SkateSessionReason.Count];
        private readonly long[] activations = new long[(int)SkateSessionReason.Count];
        private readonly long[] resets = new long[(int)SkateSessionReason.Count];
        public bool HasDisarm { get; private set; }
        public bool HasActivation { get; private set; }
        public bool HasWarmupReset { get; private set; }
        public bool HasSample { get; private set; }
        public SkateSessionObservation LastDisarm { get; private set; }
        public SkateSessionObservation LastActivation { get; private set; }
        public SkateSessionObservation LastWarmupReset { get; private set; }
        public SkateSessionObservation LastSample { get; private set; }

        private static bool Valid(SkateSessionReason reason)
        { return reason > SkateSessionReason.None && reason < SkateSessionReason.Count; }
        private static void Increment(long[] counts, SkateSessionReason reason)
        { if (Valid(reason) && counts[(int)reason] < long.MaxValue) counts[(int)reason]++; }
        public long DisarmCount(SkateSessionReason reason)
        { return Valid(reason) ? disarms[(int)reason] : 0; }
        public long ActivationCount(SkateSessionReason reason)
        { return Valid(reason) ? activations[(int)reason] : 0; }
        public long WarmupResetCount(SkateSessionReason reason)
        { return Valid(reason) ? resets[(int)reason] : 0; }

        public void Sample(SkateSessionObservation observation)
        { LastSample = observation; HasSample = true; }
        public void Disarm(SkateSessionObservation observation)
        {
            Sample(observation);
            if (!observation.WasActive || !Valid(observation.Reason)) return;
            LastDisarm = observation; HasDisarm = true;
            Increment(disarms, observation.Reason);
        }
        public void WarmupReset(SkateSessionObservation observation)
        {
            Sample(observation);
            if (observation.WasActive || !Valid(observation.Reason) ||
                !SkateSessionBounds.Finite(observation.RearmSeconds) || observation.RearmSeconds <= 0) return;
            LastWarmupReset = observation; HasWarmupReset = true;
            Increment(resets, observation.Reason);
        }
        public void Activate(SkateSessionObservation observation)
        {
            if (observation.Reason != SkateSessionReason.ManualOn &&
                observation.Reason != SkateSessionReason.AutomaticRearm) return;
            LastActivation = observation; HasActivation = true;
            Increment(activations, observation.Reason);
        }
    }

    public sealed class SkateDiagnosticsStore
    {
        public const int Capacity = 512;
        private sealed class Entry
        {
            public object Identity;
            public SkateSessionHistory History;
        }
        private readonly Dictionary<ulong, Entry> entries = new Dictionary<ulong, Entry>();
        private readonly ulong[] removals = new ulong[Capacity];
        public int Count { get { return entries.Count; } }

        public bool TryGet(ulong id, object identity, out SkateSessionHistory history)
        {
            Entry entry;
            history = null;
            if (identity == null || !entries.TryGetValue(id, out entry) ||
                !ReferenceEquals(identity, entry.Identity)) return false;
            history = entry.History; return true;
        }
        public SkateSessionHistory GetOrCreate(ulong id, object identity)
        {
            if (id == 0 || identity == null) return null;
            Entry entry;
            if (entries.TryGetValue(id, out entry))
            {
                if (ReferenceEquals(identity, entry.Identity)) return entry.History;
                entries.Remove(id);
            }
            if (Count >= Capacity) return null;
            entry = new Entry { Identity = identity, History = new SkateSessionHistory() };
            entries.Add(id, entry); return entry.History;
        }
        public void Remove(ulong id, object identity)
        {
            Entry entry;
            if (entries.TryGetValue(id, out entry) && ReferenceEquals(identity, entry.Identity)) entries.Remove(id);
        }
        public void RemoveInactive(Func<object, bool> connected)
        {
            int count = 0;
            foreach (var pair in entries)
                if (!connected(pair.Value.Identity)) removals[count++] = pair.Key;
            for (int i = 0; i < count; i++) entries.Remove(removals[i]);
        }
        public void Clear() { entries.Clear(); }
    }

    public enum SkateStatusError { None, TargetRequired, InvalidTarget, Forbidden }

    public static class SkateStatusAccess
    {
        public static bool TrySelectTarget(ulong actorId, bool hasPlayer, bool privileged, string explicitTarget,
                                          out ulong targetId, out SkateStatusError error)
        {
            targetId = 0; error = SkateStatusError.None;
            if (explicitTarget == null)
            {
                if (!hasPlayer || actorId == 0) { error = SkateStatusError.TargetRequired; return false; }
                targetId = actorId; return true;
            }
            if (!ulong.TryParse(explicitTarget, NumberStyles.None, CultureInfo.InvariantCulture, out targetId) || targetId == 0)
            { targetId = 0; error = SkateStatusError.InvalidTarget; return false; }
            if (!privileged && (!hasPlayer || targetId != actorId))
            { targetId = 0; error = SkateStatusError.Forbidden; return false; }
            return true;
        }
    }

    public static class SkateStatusReport
    {
        public static string Number(double value)
        { return SkateSessionBounds.Finite(value) ? value.ToString("G9", CultureInfo.InvariantCulture) : "unknown"; }
        private static double Age(double now, double time)
        {
            double age = now - time;
            return SkateSessionBounds.Finite(now) && SkateSessionBounds.Finite(time) && SkateSessionBounds.Finite(age) ?
                Math.Max(0, age) : double.NaN;
        }
        private static void Observation(StringBuilder text, string label, SkateSessionObservation value, double now)
        {
            text.Append(label).Append(": ").Append(value.Reason).Append("; age=")
                .Append(Number(Age(now, value.Time))).Append("s; dt=").Append(Number(value.DeltaSeconds))
                .Append("s; gap=").Append(Number(value.GapSeconds)).Append("s/0.5s; horizontal=")
                .Append(Number(value.HorizontalPath)).Append("m/").Append(Number(value.MaximumSpeed * value.DeltaSeconds))
                .Append("m; speed=").Append(Number(SkateSessionBounds.Finite(value.DeltaSeconds) && value.DeltaSeconds > 0 ? value.HorizontalPath / value.DeltaSeconds : double.NaN))
                .Append("m/s/").Append(Number(value.MaximumSpeed)).Append("m/s; vertical=").Append(Number(value.VerticalPath)).Append("m/")
                .Append(Number(40 * value.DeltaSeconds)).Append("m; supported=").Append(value.SupportKnown ? (value.FullySupported ? "true" : "false") : "unknown")
                .Append("; air=").Append(Number(value.AirSeconds)).Append("s/").Append(Number(value.MaximumAirtime))
                .Append("s; duration=").Append(Number(value.DurationSeconds)).Append("s/")
                .Append(Number(value.ElapsedSeconds + 0.125)).Append("s; warmup=").Append(Number(value.RearmSeconds))
                .Append("s/").Append(Number(value.RequiredRearmSeconds)).Append("s; position-gap=")
                .Append(Number(value.PositionGap)).Append("m/0.01m; batch=").Append(value.Batch.ToString(CultureInfo.InvariantCulture))
                .Append("; segments=").Append(value.SegmentCount.ToString(CultureInfo.InvariantCulture)).Append('/').Append(value.BufferSize.ToString(CultureInfo.InvariantCulture)).AppendLine();
        }
        public static string Format(SkateSessionHistory history, double now)
        {
            if (history == null) return "Diagnostics unavailable: the bounded connection store is full.";
            var text = new StringBuilder();
            if (history.HasSample) Observation(text, "Latest observation", history.LastSample, now);
            if (history.HasDisarm) Observation(text, "Last active disarm", history.LastDisarm, now);
            else text.AppendLine("Last active disarm: none recorded for this connection.");
            if (history.HasActivation) Observation(text, "Last successful activation", history.LastActivation, now);
            else text.AppendLine("Last successful activation: none recorded for this connection.");
            if (history.HasWarmupReset) Observation(text, "Last lost warmup", history.LastWarmupReset, now);
            text.Append("Counters (active disarms / successful activations / lost warmups):");
            bool any = false;
            for (SkateSessionReason reason = SkateSessionReason.None + 1; reason < SkateSessionReason.Count; reason++)
            {
                long d = history.DisarmCount(reason), a = history.ActivationCount(reason), r = history.WarmupResetCount(reason);
                if (d == 0 && a == 0 && r == 0) continue;
                any = true;
                text.Append(' ').Append(reason).Append('=').Append(d.ToString(CultureInfo.InvariantCulture)).Append('/').Append(a.ToString(CultureInfo.InvariantCulture)).Append('/').Append(r.ToString(CultureInfo.InvariantCulture)).Append(';');
            }
            if (!any) text.Append(" none.");
            return text.ToString();
        }
    }

    public sealed class SkateSessionBounds
    {
        public const double MaximumBatchSeconds = 0.5;
        public bool Active { get; private set; }
        public double AirSeconds { get; private set; }
        public SkateSessionObservation LastObservation { get; private set; }
        private double started;
        private double lastSample;
        private double lastSupported;
        private double duration;

        public static bool Eligible(bool connected, bool npc, bool alive, bool sleeping,
                                    bool wounded, bool mounted, bool swimming)
        { return connected && !npc && alive && !sleeping && !wounded && !mounted && !swimming; }

        public bool Begin(double now, bool supported)
        {
            SkateSessionObservation value = SkateSessionObservation.Empty(now, SkateSessionReason.None);
            value.FullySupported = supported; value.SupportKnown = true; value.WasActive = Active;
            value.AirSeconds = AirSeconds; value.DurationSeconds = duration; value.ElapsedSeconds = now - started;
            if (!Finite(now) || now < 0) value.Reason = SkateSessionReason.InvalidClock;
            else if (!supported) value.Reason = SkateSessionReason.Unsupported;
            else if (Active) value.Reason = SkateSessionReason.BeginRejected;
            LastObservation = value;
            if (value.Reason != SkateSessionReason.None) return false;
            Active = true;
            started = lastSample = lastSupported = now;
            duration = AirSeconds = 0;
            value.AirSeconds = value.DurationSeconds = value.ElapsedSeconds = 0;
            LastObservation = value;
            return true;
        }

        public void End() { Active = false; AirSeconds = 0; }

        public bool Observe(double now, double dt, double horizontalPath, double verticalPath,
                            bool fullySupported, double maxSpeed, double maxAir)
        {
            SkateSessionObservation value = SkateSessionObservation.Empty(now, SkateSessionReason.None);
            value.DeltaSeconds = dt; value.GapSeconds = now - lastSample;
            value.HorizontalPath = horizontalPath; value.VerticalPath = verticalPath;
            value.MaximumSpeed = maxSpeed; value.MaximumAirtime = maxAir;
            value.FullySupported = fullySupported; value.SupportKnown = true; value.WasActive = Active;
            value.AirSeconds = AirSeconds; value.DurationSeconds = duration + dt; value.ElapsedSeconds = now - started;
            if (!Active) value.Reason = SkateSessionReason.Inactive;
            else if (!Finite(now)) value.Reason = SkateSessionReason.InvalidClock;
            else if (now < lastSample) value.Reason = SkateSessionReason.BackwardsClock;
            else if (now - lastSample > MaximumBatchSeconds) value.Reason = SkateSessionReason.BatchGap;
            else if (!Finite(dt) || dt <= 0 || dt > MaximumBatchSeconds) value.Reason = SkateSessionReason.InvalidDelta;
            else if (!Finite(horizontalPath) || horizontalPath < 0) value.Reason = SkateSessionReason.InvalidHorizontalPath;
            else if (!Finite(verticalPath) || verticalPath < 0) value.Reason = SkateSessionReason.InvalidVerticalPath;
            else if (!Finite(maxSpeed) || maxSpeed <= 0) value.Reason = SkateSessionReason.InvalidMaximumSpeed;
            else if (!Finite(maxAir) || maxAir <= 0) value.Reason = SkateSessionReason.InvalidMaximumAirtime;
            else if (horizontalPath > maxSpeed * dt) value.Reason = SkateSessionReason.HorizontalSpeed;
            else if (verticalPath > 40 * dt) value.Reason = SkateSessionReason.VerticalLimit;
            else if (duration + dt > now - started + 0.125) value.Reason = SkateSessionReason.Duration;
            if (value.Reason != SkateSessionReason.None)
            {
                LastObservation = value;
                if (Active) End();
                return false;
            }
            double air = AirSeconds;
            if (!fullySupported || air > 0) air = Math.Max(air + dt, now - lastSupported);
            value.AirSeconds = air;
            if (air > maxAir)
            { value.Reason = SkateSessionReason.Airtime; LastObservation = value; End(); return false; }
            duration += dt;
            lastSample = now;
            AirSeconds = fullySupported ? 0 : air;
            if (fullySupported) lastSupported = now;
            LastObservation = value;
            return true;
        }

        public static bool Finite(double value)
        { return !double.IsNaN(value) && !double.IsInfinity(value); }
    }

    public sealed class SkateSessionRearm
    {
        public double Seconds { get; private set; }
        public SkateSessionObservation LastObservation { get; private set; }
        private double started;
        private double last;
        private bool observing;

        public void Reset() { Seconds = 0; observing = false; }

        public bool Observe(double now, double dt, double horizontalPath, double verticalPath,
                            bool supported, double maxSpeed, double requiredSeconds)
        {
            SkateSessionObservation value = SkateSessionObservation.Empty(now, SkateSessionReason.None);
            value.DeltaSeconds = dt; value.GapSeconds = observing ? now - last : double.NaN;
            value.HorizontalPath = horizontalPath; value.VerticalPath = verticalPath;
            value.MaximumSpeed = maxSpeed; value.FullySupported = supported; value.SupportKnown = true;
            value.RearmSeconds = Seconds; value.RequiredRearmSeconds = requiredSeconds;
            value.DurationSeconds = Seconds + dt; value.ElapsedSeconds = observing ? now - started : dt;
            if (!SkateSessionBounds.Finite(now) || now < 0) value.Reason = SkateSessionReason.InvalidClock;
            else if (!SkateSessionBounds.Finite(dt) || dt <= 0 || dt > SkateSessionBounds.MaximumBatchSeconds) value.Reason = SkateSessionReason.InvalidDelta;
            else if (!SkateSessionBounds.Finite(horizontalPath) || horizontalPath < 0) value.Reason = SkateSessionReason.InvalidHorizontalPath;
            else if (!SkateSessionBounds.Finite(verticalPath) || verticalPath < 0) value.Reason = SkateSessionReason.InvalidVerticalPath;
            else if (!supported) value.Reason = SkateSessionReason.Unsupported;
            else if (!SkateSessionBounds.Finite(maxSpeed) || maxSpeed <= 0) value.Reason = SkateSessionReason.InvalidMaximumSpeed;
            else if (!SkateSessionBounds.Finite(requiredSeconds) || requiredSeconds < 0.5 || requiredSeconds > 10) value.Reason = SkateSessionReason.InvalidRearmSeconds;
            else if (horizontalPath > maxSpeed * dt) value.Reason = SkateSessionReason.HorizontalSpeed;
            else if (verticalPath > 40 * dt) value.Reason = SkateSessionReason.VerticalLimit;
            else if (now < dt) value.Reason = SkateSessionReason.ClockBeforeDelta;
            if (value.Reason != SkateSessionReason.None)
            { LastObservation = value; Reset(); return false; }
            if (!observing)
            { started = now - dt; last = now; observing = true; }
            else if (now <= last) value.Reason = SkateSessionReason.NonIncreasingClock;
            else if (now - last > SkateSessionBounds.MaximumBatchSeconds) value.Reason = SkateSessionReason.BatchGap;
            else if (Seconds + dt > now - started + 0.125) value.Reason = SkateSessionReason.Duration;
            if (value.Reason != SkateSessionReason.None)
            { LastObservation = value; Reset(); return false; }
            Seconds += dt;
            last = now;
            value.RearmSeconds = Seconds;
            LastObservation = value;
            return Seconds + 0.000000001 >= requiredSeconds;
        }
    }

    public struct SkatePracticePart
    {
        public float Right, Forward, Height, Pitch;
    }

    public static class SkatePracticeLayout
    {
        public const string FloorPrefab = "assets/prefabs/building core/floor/floor.prefab";
        public const int PartCount = 8;

        public static bool TryPart(int index, out SkatePracticePart part)
        {
            part = new SkatePracticePart();
            switch (index)
            {
                case 0: part = new SkatePracticePart { Right = -4.5f, Forward = -3, Height = 0.39f, Pitch = -15 }; break;
                case 1: part = new SkatePracticePart { Right = 0, Forward = -3, Height = 0.75f, Pitch = -30 }; break;
                case 2: part = new SkatePracticePart { Right = 4.5f, Forward = -3, Height = 0.312f, Pitch = -12 }; break;
                case 3: part = new SkatePracticePart { Right = -4.5f, Forward = 1.5f, Height = 0.4f }; break;
                case 4: part = new SkatePracticePart { Right = 0, Forward = 1.5f, Height = 0.8f }; break;
                case 5: part = new SkatePracticePart { Right = -3, Forward = 6, Height = 0.4f }; break;
                case 6: part = new SkatePracticePart { Right = 0, Forward = 6, Height = 0.4f }; break;
                case 7: part = new SkatePracticePart { Right = 3, Forward = 6, Height = 0.4f }; break;
                default: return false;
            }
            return true;
        }
    }
}

namespace Oxide.Plugins
{
    [Info("ShortcutSkate", "Dj-Shortcut", "0.2.1")]
    [Description("Player-scoped skateboard speed and short-airtime movement bounds.")]
    public sealed class ShortcutSkate : RustPlugin
    {
        private const string UsePermission = "shortcutskate.use";
        private static ShortcutSkate instance;
        private readonly Stopwatch clock = Stopwatch.StartNew();
        private readonly Dictionary<ulong, Session> sessions = new Dictionary<ulong, Session>();
        private readonly Dictionary<ulong, BasePlayer> optedOut = new Dictionary<ulong, BasePlayer>();
        private readonly List<BaseEntity> practice = new List<BaseEntity>();
        private readonly SkateDiagnosticsStore diagnostics = new SkateDiagnosticsStore();
        private static readonly Func<object, bool> ConnectedIdentity = identity =>
        {
            BasePlayer player = identity as BasePlayer;
            return player != null && player.IsConnected;
        };
        private SkateSessionReason unavailableReason = SkateSessionReason.Unavailable;
        private Settings settings;
        private int groundMask;
        private long batch;
        private bool validating;
        private bool ready;

        public sealed class Settings
        {
            [JsonProperty(Required = Required.Always)] public int Version;
            [JsonProperty(Required = Required.Always)] public bool AllowEveryone;
            [JsonProperty(Required = Required.Always)] public double MaximumHorizontalSpeed;
            [JsonProperty(Required = Required.Always)] public double MaximumAirtime;
            [JsonProperty(Required = Required.DisallowNull)] public bool AutomaticEligiblePlayers = true;
            [JsonProperty(Required = Required.DisallowNull)] public double AutomaticRearmSeconds = 1.5;
        }

        private sealed class Session
        {
            public BasePlayer Player;
            public readonly SkateSessionBounds Bounds = new SkateSessionBounds();
            public readonly SkateSessionRearm Rearm = new SkateSessionRearm();
            public bool Manual;
            public bool HasEnd;
            public long Batch;
            public bool Allowed;
            public Vector3 LastEnd;
            public double LastObserved;
            public SkateSessionHistory History;
            public SkateSessionObservation Observation;
        }

        protected override void LoadDefaultConfig()
        {
            Config.WriteObject(new Settings { Version = 1, AllowEveryone = true,
                MaximumHorizontalSpeed = 14, MaximumAirtime = 1 }, true);
        }

        protected override void LoadConfig()
        {
            ready = false;
            unavailableReason = SkateSessionReason.Unavailable;
            settings = null;
            try
            {
                base.LoadConfig();
                settings = ParseSettings(File.ReadAllText(Config.Filename));
            }
            catch (Exception error)
            { PrintError("Skating is disabled; configuration is preserved. " + error.Message); }
        }

        public static Settings ParseSettings(string json)
        {
            var value = JsonConvert.DeserializeObject<Settings>(json, new JsonSerializerSettings
            {
                TypeNameHandling = TypeNameHandling.None,
                MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
                MissingMemberHandling = MissingMemberHandling.Error
            });
            if (value == null || value.Version != 1 ||
                !SkateSessionBounds.Finite(value.MaximumHorizontalSpeed) ||
                value.MaximumHorizontalSpeed < 1 || value.MaximumHorizontalSpeed > 50 ||
                !SkateSessionBounds.Finite(value.MaximumAirtime) ||
                value.MaximumAirtime < 0.1 || value.MaximumAirtime > 2 ||
                !SkateSessionBounds.Finite(value.AutomaticRearmSeconds) ||
                value.AutomaticRearmSeconds < 0.5 || value.AutomaticRearmSeconds > 10)
                throw new InvalidDataException("Use Version 1, speed 1-50 m/s, airtime 0.1-2 seconds and rearm 0.5-10 seconds.");
            return value;
        }

        private void Init()
        {
            permission.RegisterPermission(UsePermission, this);
            instance = this;
            groundMask = LayerMask.GetMask("Terrain", "World", "Construction", "Deployed");
        }

        private void OnServerInitialized()
        {
            ready = settings != null && groundMask != 0 && Patched("ValidateMoves", typeof(MovePatch), false) &&
                Patched("AreSpeeding", typeof(SpeedPatch), true) && Patched("AreFlying", typeof(FlyPatch), true);
            if (!ready) PrintError("Skating is disabled: matching movement patches and valid config are required.");
            timer.Every(0.1f, Sweep);
        }

        private static bool Patched(string name, Type patch, bool postfix)
        {
            var method = AccessTools.Method(typeof(AntiHack), name);
            var info = method == null ? null : Harmony.GetPatchInfo(method);
            if (info == null) return false;
            return (postfix ? info.Postfixes : info.Prefixes).Any(p => p.PatchMethod.DeclaringType == patch) &&
                (postfix || info.Finalizers.Any(p => p.PatchMethod.DeclaringType == patch));
        }

        private bool Permitted(BasePlayer player)
        { return settings != null && (settings.AllowEveryone || permission.UserHasPermission(player.UserIDString, UsePermission)); }

        private static bool Eligible(BasePlayer player)
        {
            return player != null && SkateSessionBounds.Eligible(player.IsConnected, player.IsNpc,
                player.IsAlive(), player.IsSleeping(), player.IsWounded(), player.isMounted, player.IsSwimming()) &&
                player.GetParentEntity() == null;
        }

        private bool OptedOut(BasePlayer player)
        {
            BasePlayer previous;
            if (!optedOut.TryGetValue(player.userID, out previous)) return false;
            if (ReferenceEquals(previous, player)) return true;
            optedOut.Remove(player.userID);
            return false;
        }

        [ConsoleCommand("skate.toggle")]
        private void Toggle(ConsoleSystem.Arg arg) { Command(arg, 0); }
        [ConsoleCommand("skate.on")]
        private void On(ConsoleSystem.Arg arg) { Command(arg, 1); }
        [ConsoleCommand("skate.off")]
        private void Off(ConsoleSystem.Arg arg) { Command(arg, -1); }
        [ConsoleCommand("skate.status")]
        private void Status(ConsoleSystem.Arg arg)
        {
            int count = arg.Args == null ? 0 : arg.Args.Length;
            if (count > 1) { arg.ReplyWith("Usage: skate.status [steamID]."); return; }
            BasePlayer actor = arg.Player();
            bool privileged = (actor != null && actor.IsAdmin) || arg.IsAdmin ||
                (actor == null && arg.IsServerside && arg.Connection == null);
            ulong target;
            SkateStatusError error;
            if (!SkateStatusAccess.TrySelectTarget(actor == null ? 0 : actor.userID, actor != null,
                privileged, count == 0 ? null : arg.GetString(0), out target, out error))
            {
                arg.ReplyWith(error == SkateStatusError.Forbidden ? "You may only view your own skating status." :
                    error == SkateStatusError.TargetRequired ? "Usage: skate.status <steamID>." : "Use one nonzero numeric Steam ID.");
                return;
            }
            // Authorize the explicit ID before consulting the native player cache.
            BasePlayer player = actor != null && actor.userID == target ? actor : BasePlayer.FindByID(target);
            if (player == null || !player.IsConnected || player.IsNpc)
            { arg.ReplyWith("No connected player with that Steam ID."); return; }
            Session session;
            bool candidate = sessions.TryGetValue(player.userID, out session) && ReferenceEquals(player, session.Player);
            if (candidate && (!ready || !Eligible(player) || !Permitted(player) || OptedOut(player)))
            { Stop(player, PolicyReason(player)); candidate = false; }
            string state = candidate ? (session.Bounds.Active ? "Skating on." : "Skating waiting for supported movement.") :
                (OptedOut(player) ? "Skating off; automatic skating is disabled for this connection." : "Skating off.");
            SkateSessionHistory history;
            if (!diagnostics.TryGet(player.userID, player, out history))
                history = diagnostics.GetOrCreate(player.userID, player);
            if (candidate) session.History = history;
            arg.ReplyWith(state + "\nSteam ID " + target.ToString(CultureInfo.InvariantCulture) +
                "; ready=" + ready + "; unavailable-reason=" + (ready ? SkateSessionReason.None : unavailableReason) + "; mode=" + (candidate ? (session.Manual ? "manual" : "automatic") : "inactive") +
                "; last-batch-allowed=" + (candidate && session.Allowed) +
                "; speed-limit=" + SkateStatusReport.Number(settings == null ? double.NaN : settings.MaximumHorizontalSpeed) +
                "m/s; air-limit=" + SkateStatusReport.Number(settings == null ? double.NaN : settings.MaximumAirtime) +
                "s; automatic=" + (settings != null && settings.AutomaticEligiblePlayers) +
                "; rearm-required=" + SkateStatusReport.Number(settings == null ? double.NaN : settings.AutomaticRearmSeconds) +
                "s.\n" + SkateStatusReport.Format(history, clock.Elapsed.TotalSeconds));
        }

        private void Command(ConsoleSystem.Arg arg, int action)
        {
            BasePlayer player = arg.Player();
            if (player == null) { arg.ReplyWith("Use this command as a player."); return; }
            Session session;
            bool candidate = sessions.TryGetValue(player.userID, out session) && ReferenceEquals(player, session.Player);
            if (action < 0 || (action == 0 && candidate))
            {
                optedOut[player.userID] = player;
                Stop(player, SkateSessionReason.ManualOff);
                arg.ReplyWith("Skating off; automatic skating is disabled for this connection.");
                return;
            }
            if (!ready) { arg.ReplyWith("Skating is unavailable. Ask the server administrator."); return; }
            if (!Permitted(player)) { arg.ReplyWith("You do not have permission to skate."); return; }
            if (!Eligible(player) || !Supported(player.transform.position))
            { arg.ReplyWith("Stand on dry ground, alive and awake, to skate."); return; }
            optedOut.Remove(player.userID);
            if (candidate && session.Bounds.Active) { session.Manual = true; arg.ReplyWith("Skating on."); return; }
            Stop(player, SkateSessionReason.ManualRestart);
            session = new Session { Player = player, Manual = true, HasEnd = true, LastEnd = player.transform.position,
                History = diagnostics.GetOrCreate(player.userID, player) };
            session.LastObserved = clock.Elapsed.TotalSeconds;
            if (!session.Bounds.Begin(session.LastObserved, true)) return;
            sessions[player.userID] = session;
            SkateSessionObservation activation = session.Bounds.LastObservation;
            activation.Reason = SkateSessionReason.ManualOn;
            activation.MaximumSpeed = settings.MaximumHorizontalSpeed;
            activation.MaximumAirtime = settings.MaximumAirtime;
            activation.RearmSeconds = 0; activation.RequiredRearmSeconds = settings.AutomaticRearmSeconds;
            session.Observation = activation;
            if (session.History != null) { session.History.Sample(activation); session.History.Activate(activation); }
            arg.ReplyWith("Skating on.");
        }

        private SkateSessionReason PolicyReason(BasePlayer player)
        {
            if (!ready) return unavailableReason;
            if (!Eligible(player))
            {
                if (player == null || !player.IsConnected) return SkateSessionReason.Disconnected;
                if (player.IsNpc) return SkateSessionReason.Npc;
                if (!player.IsAlive()) return SkateSessionReason.Dead;
                if (player.IsSleeping()) return SkateSessionReason.Sleeping;
                if (player.IsWounded()) return SkateSessionReason.Wounded;
                if (player.isMounted) return SkateSessionReason.Mounted;
                if (player.IsSwimming()) return SkateSessionReason.Swimming;
                return SkateSessionReason.Parented;
            }
            if (!Permitted(player)) return SkateSessionReason.PermissionDenied;
            if (OptedOut(player)) return SkateSessionReason.OptedOut;
            return SkateSessionReason.None;
        }

        private SkateSessionObservation Snapshot(Session session, SkateSessionReason reason, double now)
        {
            SkateSessionObservation value = session.Observation;
            value.Reason = reason;
            value.ElapsedSeconds += now - value.Time;
            value.Time = now; value.GapSeconds = now - session.LastObserved;
            value.WasActive = session.Bounds.Active; value.AirSeconds = session.Bounds.AirSeconds;
            value.RearmSeconds = session.Rearm.Seconds;
            value.MaximumSpeed = settings == null ? double.NaN : settings.MaximumHorizontalSpeed;
            value.MaximumAirtime = settings == null ? double.NaN : settings.MaximumAirtime;
            value.RequiredRearmSeconds = settings == null ? double.NaN : settings.AutomaticRearmSeconds;
            return value;
        }

        private static SkateSessionObservation Context(SkateSessionObservation value, SkateSessionObservation batchValue)
        {
            value.Batch = batchValue.Batch; value.SegmentCount = batchValue.SegmentCount;
            value.BufferSize = batchValue.BufferSize; value.PositionGap = batchValue.PositionGap;
            value.RequiredRearmSeconds = batchValue.RequiredRearmSeconds;
            value.MaximumAirtime = batchValue.MaximumAirtime;
            return value;
        }

        private void Stop(BasePlayer player, SkateSessionReason reason)
        {
            if (ReferenceEquals(player, null)) return;
            Session session;
            if (!sessions.TryGetValue(player.userID, out session) || !ReferenceEquals(player, session.Player)) return;
            Disarm(session, Snapshot(session, reason, clock.Elapsed.TotalSeconds));
            sessions.Remove(player.userID);
        }

        private static void Disarm(Session session, SkateSessionObservation observation)
        {
            session.Observation = observation;
            if (session.History != null)
            {
                session.History.Disarm(observation);
                session.History.WarmupReset(observation);
            }
            session.Bounds.End();
            session.Rearm.Reset();
            session.Allowed = false;
            session.HasEnd = false;
        }

        private void Sweep()
        {
            foreach (Session session in sessions.Values.ToArray())
            {
                if (!ready || !Eligible(session.Player) || !Permitted(session.Player) || OptedOut(session.Player))
                    Stop(session.Player, PolicyReason(session.Player));
                else if (session.Bounds.Active && clock.Elapsed.TotalSeconds - session.LastObserved > SkateSessionBounds.MaximumBatchSeconds)
                    Disarm(session, Snapshot(session, batch == long.MaxValue ? SkateSessionReason.BatchExhausted :
                        SkateSessionReason.BatchGap, clock.Elapsed.TotalSeconds));
            }
            foreach (var pair in optedOut.ToArray())
                if (pair.Value == null || !pair.Value.IsConnected) optedOut.Remove(pair.Key);
            diagnostics.RemoveInactive(ConnectedIdentity);
        }

        private void OnPlayerDeath(BasePlayer player, HitInfo info) { Stop(player, SkateSessionReason.Dead); }
        private void OnPlayerSleep(BasePlayer player) { Stop(player, SkateSessionReason.Sleeping); }
        private void OnPlayerDisconnected(BasePlayer player, string reason)
        {
            Stop(player, SkateSessionReason.Disconnected);
            BasePlayer previous;
            if (player != null)
            {
                diagnostics.Remove(player.userID, player);
                if (optedOut.TryGetValue(player.userID, out previous) && ReferenceEquals(player, previous))
                    optedOut.Remove(player.userID);
            }
        }
        private void OnEntityMounted(BaseMountable mountable, BasePlayer player) { Stop(player, SkateSessionReason.Mounted); }
        private void OnPlayerWound(BasePlayer player, HitInfo info) { Stop(player, SkateSessionReason.Wounded); }

        private void Unload()
        {
            ready = false;
            foreach (Session session in sessions.Values.ToArray()) Stop(session.Player, SkateSessionReason.Unload);
            optedOut.Clear();
            diagnostics.Clear();
            ClearPractice();
            if (practice.Count != 0) PrintError("Unload could not remove " + practice.Count + " practice parts; an entity kill hook or server error prevented cleanup.");
            if (ReferenceEquals(instance, this)) instance = null;
        }

        [ConsoleCommand("skate.practice.spawn")]
        private void SpawnPractice(ConsoleSystem.Arg arg)
        {
            BasePlayer player = arg.Player();
            if (player == null || !player.IsAdmin) { arg.ReplyWith("Use this command as an in-game administrator."); return; }
            if (!ready || !Eligible(player)) { arg.ReplyWith("Practice is unavailable; stand alive on dry ground with skating ready."); return; }
            if (practice.Count != 0) { arg.ReplyWith("Clear the existing practice area first with skate.practice.clear."); return; }
            try
            {
                string path = SkatePracticeLayout.FloorPrefab;
                var manifest = GameManifest.Current;
                var prefab = GameManager.server.FindPrefab(path);
                var construction = PrefabAttribute.server.Find<Construction>(StringPool.Get(path));
                var stone = construction == null ? null : construction.GetGrade(BuildingGrade.Enum.Stone, 0);
                if (manifest == null || manifest.entities == null ||
                    !manifest.entities.Any(p => string.Equals(p, path, StringComparison.OrdinalIgnoreCase)) ||
                    prefab == null || prefab.GetComponent<BuildingBlock>() == null || construction == null ||
                    stone == null || stone.gradeBase == null || stone.gradeBase.type != BuildingGrade.Enum.Stone)
                { arg.ReplyWith("Practice floor prefab or stone construction data is unavailable in this server build."); return; }
                Vector3 forward = player.eyes.BodyForward();
                forward.y = 0;
                if (!Coordinate(forward) || forward.sqrMagnitude < 0.0001f)
                { arg.ReplyWith("Look horizontally toward an open practice area."); return; }
                forward.Normalize();
                Vector3 right = Vector3.Cross(Vector3.up, forward);
                Vector3 origin = player.transform.position + forward * 12;
                RaycastHit ground;
                int terrainMask = LayerMask.GetMask("Terrain", "World");
                if (!Physics.Raycast(origin + Vector3.up * 3, Vector3.down, out ground, 6, terrainMask, QueryTriggerInteraction.Ignore))
                { arg.ReplyWith("Practice needs open, flat dry ground about twelve metres ahead."); return; }
                origin.y = ground.point.y;
                for (int x = -4; x <= 4; x++)
                    for (int z = -3; z <= 5; z++)
                    {
                        Vector3 point = origin + right * (x * 1.5f) + forward * (z * 1.5f);
                        if (!Physics.Raycast(point + Vector3.up * 3, Vector3.down, out ground, 6, terrainMask, QueryTriggerInteraction.Ignore) ||
                            ground.normal.y < 0.98f || Math.Abs(ground.point.y - origin.y) > 0.1f ||
                            WaterLevel.Test(ground.point + Vector3.up * 0.1f, true, false))
                        { arg.ReplyWith("Practice needs a clear, flat dry area about fourteen metres wide."); return; }
                    }
                Quaternion yaw = Quaternion.LookRotation(forward, Vector3.up);
                if (Physics.CheckBox(origin + forward * 1.5f + Vector3.up * 1.5f,
                    new Vector3(7, 1.4f, 7.5f), yaw,
                    LayerMask.GetMask("Construction", "Deployed", "Player (Server)"), QueryTriggerInteraction.Ignore))
                { arg.ReplyWith("Practice space is occupied by a player or existing building."); return; }
                for (int index = 0; index < SkatePracticeLayout.PartCount; index++)
                {
                    SkatePracticePart part;
                    if (!SkatePracticeLayout.TryPart(index, out part)) throw new InvalidDataException("Invalid practice layout.");
                    Vector3 position = origin + right * part.Right + forward * part.Forward + Vector3.up * part.Height;
                    BaseEntity entity = GameManager.server.CreateEntity(path, position, yaw * Quaternion.Euler(part.Pitch, 0, 0), true);
                    if (entity == null) throw new InvalidDataException("Practice floor creation failed.");
                    practice.Add(entity);
                    BuildingBlock block = entity as BuildingBlock;
                    if (block == null) throw new InvalidDataException("Practice floor is not a building part.");
                    block.enableSaving = false;
                    block.CullBushes = false;
                    block.grounded = true;
                    block.OwnerID = player.userID;
                    block.Spawn();
                    if (block.IsDestroyed || block.blockDefinition == null) throw new InvalidDataException("Practice floor initialization failed.");
                    block.SetGrade(BuildingGrade.Enum.Stone);
                    block.SetHealthToMax();
                    block.SendNetworkUpdateImmediate();
                }
                arg.ReplyWith("Practice area spawned: two ramps, a bank, 0.4 m and 0.8 m ledges, and a long low ledge. Clear with skate.practice.clear.");
            }
            catch (Exception error)
            {
                ClearPractice();
                PrintError("Practice creation failed. " + error.Message);
                arg.ReplyWith(practice.Count == 0 ? "Practice could not be created; created parts were removed." :
                    "Practice creation failed and cleanup is incomplete; use skate.practice.clear.");
            }
        }

        [ConsoleCommand("skate.practice.clear")]
        private void PracticeClear(ConsoleSystem.Arg arg)
        {
            BasePlayer player = arg.Player();
            if (player == null || !player.IsAdmin) { arg.ReplyWith("Use this command as an in-game administrator."); return; }
            ClearPractice();
            arg.ReplyWith(practice.Count == 0 ? "Practice area cleared." : "Practice cleanup is incomplete; check the server log and retry.");
        }

        private void ClearPractice()
        {
            for (int i = practice.Count - 1; i >= 0; i--)
            {
                BaseEntity entity = practice[i];
                try
                {
                    if (entity != null && !entity.IsDestroyed) entity.Kill();
                    if (entity != null && !entity.IsDestroyed)
                    { PrintError("Practice part removal was refused by the server; the part remains tracked."); continue; }
                    practice.RemoveAt(i);
                }
                catch (Exception error) { PrintError("Practice part cleanup failed. " + error.Message); }
            }
        }

        private bool Supported(Vector3 point)
        {
            RaycastHit hit;
            return Physics.Raycast(point + Vector3.up * 0.15f, Vector3.down, out hit, 0.3f,
                groundMask, QueryTriggerInteraction.Ignore) && hit.normal.y >= 0.70710677f;
        }

        private void BeginBatch(ref BasePlayer.PlayerServerStates.ReadOnly states, NativeArray<int>.ReadOnly indices)
        {
            if (!ready || validating || batch == long.MaxValue) { validating = false; return; }
            validating = true;
            batch++;
            double now = clock.Elapsed.TotalSeconds;
            foreach (int index in indices)
            {
                BasePlayer player = states.PlayerCache.Objects[index];
                if (player == null) continue;
                Session session;
                if (sessions.TryGetValue(player.userID, out session) && !ReferenceEquals(player, session.Player))
                {
                    Disarm(session, Snapshot(session, SkateSessionReason.IdentityChanged, now));
                    diagnostics.Remove(player.userID, session.Player);
                    sessions.Remove(player.userID); session = null;
                }
                if (player.ActivePlayerInd != index || !Eligible(player) || !Permitted(player) ||
                    OptedOut(player) || states.CachedStates[index].IsSwimming)
                {
                    SkateSessionReason reason = player.ActivePlayerInd != index ? SkateSessionReason.CacheIndexMismatch : PolicyReason(player);
                    if (reason == SkateSessionReason.None) reason = SkateSessionReason.Swimming;
                    Stop(player, reason); continue;
                }
                if (session == null)
                {
                    if (!settings.AutomaticEligiblePlayers) continue;
                    session = new Session { Player = player, History = diagnostics.GetOrCreate(player.userID, player),
                        Observation = SkateSessionObservation.Empty(now, SkateSessionReason.None) };
                    sessions[player.userID] = session;
                }
                if (session.History == null) session.History = diagnostics.GetOrCreate(player.userID, player);
                session.Allowed = false;
                session.Batch = batch;
                if (!session.Manual && !settings.AutomaticEligiblePlayers) { Stop(player, SkateSessionReason.AutomaticDisabled); continue; }
                SkateSessionObservation value = Snapshot(session, SkateSessionReason.None, now);
                value.Batch = batch; value.DeltaSeconds = value.HorizontalPath = value.VerticalPath = double.NaN;
                value.SegmentCount = value.BufferSize = -1; value.PositionGap = double.NaN; value.SupportKnown = false;
                try
                {
                    var cache = states.TickCache;
                    int count = cache.Infos[index].Count;
                    value.SegmentCount = count; value.BufferSize = cache.BufferSize;
                    if (count < 1 || count >= cache.BufferSize)
                    { value.Reason = SkateSessionReason.InvalidTickCount; Disarm(session, value); continue; }
                    int offset = checked(index * cache.BufferSize);
                    double dt = states.TickDeltaTime[index];
                    value.DeltaSeconds = dt;
                    if (!SkateSessionBounds.Finite(dt) || dt <= 0 || dt > SkateSessionBounds.MaximumBatchSeconds)
                    { value.Reason = SkateSessionReason.InvalidDelta; Disarm(session, value); continue; }
                    Vector3 previous = cache.Segments[offset].point;
                    if (!Coordinate(previous))
                    { value.Reason = SkateSessionReason.InvalidCoordinate; Disarm(session, value); continue; }
                    if (session.HasEnd) value.PositionGap = Math.Sqrt((previous - session.LastEnd).sqrMagnitude);
                    if (session.HasEnd && (previous - session.LastEnd).sqrMagnitude > 0.0001f)
                    { value.Reason = SkateSessionReason.PositionDiscontinuity; Disarm(session, value); continue; }
                    bool supported = Supported(previous);
                    value.FullySupported = supported; value.SupportKnown = true;
                    double horizontal = 0, vertical = 0;
                    bool invalidCoordinate = false, supportComplete = true;
                    for (int s = 1; s <= count; s++)
                    {
                        Vector3 point = cache.Segments[offset + s].point;
                        if (!Coordinate(point)) { horizontal = double.NaN; invalidCoordinate = true; supportComplete = false; break; }
                        double x = (double)point.x - previous.x, z = (double)point.z - previous.z;
                        horizontal += Math.Sqrt(x * x + z * z);
                        vertical += Math.Abs((double)point.y - previous.y);
                        value.HorizontalPath = horizontal; value.VerticalPath = vertical;
                        if (horizontal > settings.MaximumHorizontalSpeed * dt || vertical > 40 * dt) { supportComplete = false; break; }
                        int samples = Math.Max(1, (int)Math.Ceiling((point - previous).magnitude / 0.1));
                        for (int sample = 1; sample <= samples; sample++)
                            supported = Supported(Vector3.Lerp(previous, point, (float)sample / samples)) && supported;
                        previous = point;
                    }
                    if (session.Bounds.Active)
                    {
                        session.Allowed = session.Bounds.Observe(now, dt, horizontal,
                            vertical, supported, settings.MaximumHorizontalSpeed, settings.MaximumAirtime);
                        double priorWarmup = value.RearmSeconds;
                        value = Context(session.Bounds.LastObservation, value);
                        value.SupportKnown = supportComplete;
                        value.RearmSeconds = priorWarmup;
                        if (invalidCoordinate && value.Reason == SkateSessionReason.InvalidHorizontalPath)
                            value.Reason = SkateSessionReason.InvalidCoordinate;
                        if (!session.Allowed) { Disarm(session, value); continue; }
                    }
                    else
                    {
                        bool rearmed = session.Rearm.Observe(now, dt, horizontal, vertical,
                            supported, settings.MaximumHorizontalSpeed, settings.AutomaticRearmSeconds);
                        value = Context(session.Rearm.LastObservation, value);
                        value.SupportKnown = supportComplete;
                        value.AirSeconds = session.Bounds.AirSeconds;
                        if (invalidCoordinate && value.Reason == SkateSessionReason.InvalidHorizontalPath)
                            value.Reason = SkateSessionReason.InvalidCoordinate;
                        if (session.Rearm.Seconds == 0) { Disarm(session, value); continue; }
                        if (rearmed)
                        {
                            double achievedWarmup = session.Rearm.Seconds;
                            session.Allowed = session.Bounds.Begin(now - dt, true) &&
                                session.Bounds.Observe(now, dt, horizontal, vertical, supported,
                                    settings.MaximumHorizontalSpeed, settings.MaximumAirtime);
                            value = Context(session.Bounds.LastObservation, value);
                            value.WasActive = false; value.RearmSeconds = achievedWarmup;
                            if (session.Allowed)
                            {
                                value.Reason = SkateSessionReason.AutomaticRearm;
                                if (session.History != null) session.History.Activate(value);
                            }
                            session.Rearm.Reset();
                            if (!session.Allowed) { Disarm(session, value); continue; }
                        }
                    }
                    session.Observation = value;
                    if (session.History != null) session.History.Sample(value);
                    session.HasEnd = true;
                    session.LastEnd = previous;
                    session.LastObserved = now;
                }
                catch (Exception)
                { value.Reason = SkateSessionReason.BatchException; value.SupportKnown = false; Disarm(session, value); }
            }
        }

        private static bool Coordinate(Vector3 value)
        {
            return SkateSessionBounds.Finite(value.x) && SkateSessionBounds.Finite(value.y) &&
                SkateSessionBounds.Finite(value.z) && Math.Abs(value.x) <= 100000 &&
                Math.Abs(value.y) <= 100000 && Math.Abs(value.z) <= 100000;
        }

        private void Filter(ref BasePlayer.PlayerServerStates.ReadOnly states,
                            NativeArray<int>.ReadOnly indices, NativeArray<bool> results)
        {
            if (!ready || !validating) return;
            for (int n = 0; n < indices.Length; n++)
            {
                int index = indices[n];
                BasePlayer player = states.PlayerCache.Objects[index];
                Session session;
                if (player != null && player.ActivePlayerInd == index &&
                    sessions.TryGetValue(player.userID, out session) && ReferenceEquals(player, session.Player) &&
                    session.Batch == batch && session.Allowed && session.Bounds.Active &&
                    !OptedOut(player) && Eligible(player) && Permitted(player)) results[n] = false;
            }
        }

        // The result arrays are consumed before penalties and position rejection.
        // Patch the completed batch, preserving every other anti-hack result.
        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "ValidateMoves")]
        private static class MovePatch
        {
            private static void Prefix(ref BasePlayer.PlayerServerStates.ReadOnly __0, NativeArray<int>.ReadOnly __1)
            {
                if (instance == null) return;
                try { instance.BeginBatch(ref __0, __1); }
                catch (Exception) { instance.validating = false; instance.ready = false; instance.unavailableReason = SkateSessionReason.MovePatchException; }
            }
            private static Exception Finalizer(Exception __exception)
            { if (instance != null) instance.validating = false; return __exception; }
        }

        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "AreSpeeding")]
        private static class SpeedPatch
        {
            private static void Postfix(ref BasePlayer.PlayerServerStates.ReadOnly __0,
                                        NativeArray<int>.ReadOnly __2, NativeArray<bool> __3)
            {
                if (instance == null) return;
                try { instance.Filter(ref __0, __2, __3); }
                catch (Exception) { instance.validating = false; instance.ready = false; instance.unavailableReason = SkateSessionReason.SpeedPatchException; }
            }
        }

        [AutoPatch]
        [HarmonyPatch(typeof(AntiHack), "AreFlying")]
        private static class FlyPatch
        {
            private static void Postfix(ref BasePlayer.PlayerServerStates.ReadOnly __0,
                                        NativeArray<int>.ReadOnly __3, NativeArray<bool> __4)
            {
                if (instance == null) return;
                try { instance.Filter(ref __0, __3, __4); }
                catch (Exception) { instance.validating = false; instance.ready = false; instance.unavailableReason = SkateSessionReason.FlyPatchException; }
            }
        }
    }
}
