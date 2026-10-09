using System;

namespace Shortcut.RustMod
{
    public enum RiderInputAdmission { NewObservation = 0, NoNewObservation = 1, Unavailable = 2 }

    public sealed class RiderInputObservation
    {
        public const int MaximumLifetimeTicks = 10;
        private readonly RiderControlRequest request;
        private readonly long admissionTick;
        private readonly int lifetimeTicks;

        private RiderInputObservation(RiderControlRequest request, long admissionTick, int lifetimeTicks)
        { this.request = request; this.admissionTick = admissionTick; this.lifetimeTicks = lifetimeTicks; }

        public RiderControlRequest Request { get { return request; } }
        public long AdmissionTick { get { return admissionTick; } }
        public int LifetimeTicks { get { return lifetimeTicks; } }

        public static bool TryCreate(RiderControlRequest request, long admissionTick, int lifetimeTicks,
            out RiderInputObservation observation, out string error)
        {
            observation = null;
            error = "Invalid rider input observation.";
            if (request == null || request.Identity.PlayerId == 0 ||
                request.Identity.RuntimeId == Guid.Empty || request.Identity.LifeId == Guid.Empty ||
                request.Sequence <= 0 || !ValidAxis(request.Skate.Steer) ||
                !ValidAxis(request.Skate.Spin) || !ValidAxis(request.Skate.Flip) ||
                admissionTick < 0 || admissionTick > Gunplay.MaximumTick ||
                lifetimeTicks < 1 || lifetimeTicks > MaximumLifetimeTicks) return false;
            observation = new RiderInputObservation(request, admissionTick, lifetimeTicks);
            error = null;
            return true;
        }

        // Admission slots do not establish callback provenance or wall-time freshness.
        public bool IsValidAt(long hostTick)
        {
            return hostTick >= admissionTick && hostTick <= Gunplay.MaximumTick &&
                hostTick - admissionTick < lifetimeTicks;
        }

        private static bool ValidAxis(double value)
        { return !double.IsNaN(value) && !double.IsInfinity(value) && value >= -1 && value <= 1; }
    }
}
