using System;
using System.Threading;

namespace Shortcut.RustMod
{
    // Create on the server main thread and capture only in the bound player's
    // genuine OnPlayerInput callback. The host closes before lifecycle changes.
    public sealed class RustRiderInput
    {
        private const int SupportedButtons = (int)(BUTTON.FORWARD | BUTTON.BACKWARD |
            BUTTON.LEFT | BUTTON.RIGHT | BUTTON.JUMP | BUTTON.FIRE_PRIMARY |
            BUTTON.FIRE_SECONDARY | BUTTON.RELOAD);
        private const int KnownButtons = SupportedButtons |
            (int)(BUTTON.DUCK | BUTTON.SPRINT | BUTTON.USE | BUTTON.FIRE_THIRD);

        private readonly BasePlayer rider;
        private readonly RiderIdentity identity;
        private readonly int serverThread;
        private int closed;
        private bool capturing;
        private long lastSlot = -1;
        private long sequence;

        private RustRiderInput(BasePlayer rider, RiderIdentity identity)
        {
            this.rider = rider;
            this.identity = identity;
            serverThread = Thread.CurrentThread.ManagedThreadId;
        }

        public static bool TryCreate(BasePlayer rider, RiderIdentity identity,
                                     out RustRiderInput reader, out string error)
        {
            reader = null;
            error = "Invalid rider input binding.";
            if (!ValidIdentity(identity) || ReferenceEquals(rider, null)) return false;
            try
            {
                if (!ValidRider(rider, identity.PlayerId)) return false;
                var result = new RustRiderInput(rider, identity);
                if (!ValidRider(rider, identity.PlayerId)) return false;
                reader = result;
                error = null;
                return true;
            }
            catch (Exception) { return false; }
        }

        public void Close() { Interlocked.Exchange(ref closed, 1); }

        public bool TryCapture(BasePlayer callbackPlayer, InputState input,
                               RiderIdentity currentLife, long hostSlot,
                               out RiderControlRequest request, out string error)
        {
            request = null;
            error = "Rider input is unavailable.";
            // Worker misuse must not touch Unity or expire a valid host binding.
            if (Thread.CurrentThread.ManagedThreadId != serverThread || IsClosed) return false;
            if (capturing)
            {
                Close();
                return false;
            }
            if (!ValidIdentity(currentLife) || !SameIdentity(currentLife, identity))
            {
                Close();
                return false;
            }
            if (!ReferenceEquals(callbackPlayer, rider) || ReferenceEquals(input, null)) return false;
            if (hostSlot < 0 || hostSlot > Gunplay.MaximumTick || hostSlot <= lastSlot ||
                sequence == long.MaxValue)
            {
                error = "Invalid rider input slot.";
                return false;
            }

            capturing = true;
            try
            {
                if (!ValidRider(rider, identity.PlayerId))
                {
                    Close();
                    return false;
                }
                if (!ReferenceEquals(rider.serverInput, input)) return false;
                var message = input.current;
                if (ReferenceEquals(message, null)) return false;
                int buttons = message.buttons;
                if ((buttons & ~KnownButtons) != 0)
                {
                    error = "Unknown rider input buttons.";
                    return false;
                }
                int effective = EffectiveButtons(input);
                if (!ReferenceEquals(input.current, message) || message.buttons != buttons)
                {
                    Close();
                    error = "Rider input changed during capture.";
                    return false;
                }
                // A swallowed held control is unavailable, never a false release.
                if (effective != (buttons & SupportedButtons))
                {
                    error = "A supported rider control was swallowed.";
                    return false;
                }

                GunInput gun;
                SkateInput skate;
                Decode(buttons, out gun, out skate);
                long nextSequence = sequence + 1;
                var result = new RiderControlRequest(identity, nextSequence, gun, skate);

                if (!ValidRider(rider, identity.PlayerId) ||
                    !ReferenceEquals(rider.serverInput, input) ||
                    !ReferenceEquals(input.current, message) || message.buttons != buttons ||
                    EffectiveButtons(input) != effective ||
                    !ReferenceEquals(input.current, message) || message.buttons != buttons ||
                    !ValidRider(rider, identity.PlayerId) ||
                    !ReferenceEquals(rider.serverInput, input))
                {
                    Close();
                    error = "Rider input changed during capture.";
                    return false;
                }
                // Publish and advance only after the native snapshot and revocation checks.
                if (IsClosed) return false;
                sequence = nextSequence;
                lastSlot = hostSlot;
                request = result;
                error = null;
                return true;
            }
            catch (Exception)
            {
                Close();
                return false;
            }
            finally { capturing = false; }
        }

        internal static void Decode(int buttons, out GunInput gun, out SkateInput skate)
        {
            gun = new GunInput(Held(buttons, BUTTON.FIRE_PRIMARY),
                Held(buttons, BUTTON.FIRE_SECONDARY), Held(buttons, BUTTON.RELOAD));
            double steer = (Held(buttons, BUTTON.RIGHT) ? 1 : 0) -
                (Held(buttons, BUTTON.LEFT) ? 1 : 0);
            skate = new SkateInput(Held(buttons, BUTTON.FORWARD),
                Held(buttons, BUTTON.BACKWARD), Held(buttons, BUTTON.JUMP), steer, 0, 0);
        }

        private bool IsClosed { get { return Volatile.Read(ref closed) != 0; } }

        private static int EffectiveButtons(InputState input)
        {
            int buttons = 0;
            if (input.IsDown(BUTTON.FORWARD)) buttons |= (int)BUTTON.FORWARD;
            if (input.IsDown(BUTTON.BACKWARD)) buttons |= (int)BUTTON.BACKWARD;
            if (input.IsDown(BUTTON.LEFT)) buttons |= (int)BUTTON.LEFT;
            if (input.IsDown(BUTTON.RIGHT)) buttons |= (int)BUTTON.RIGHT;
            if (input.IsDown(BUTTON.JUMP)) buttons |= (int)BUTTON.JUMP;
            if (input.IsDown(BUTTON.FIRE_PRIMARY)) buttons |= (int)BUTTON.FIRE_PRIMARY;
            if (input.IsDown(BUTTON.FIRE_SECONDARY)) buttons |= (int)BUTTON.FIRE_SECONDARY;
            if (input.IsDown(BUTTON.RELOAD)) buttons |= (int)BUTTON.RELOAD;
            return buttons;
        }

        private static bool Held(int buttons, BUTTON button)
        { return (buttons & (int)button) != 0; }

        private static bool ValidRider(BasePlayer player, ulong playerId)
        {
            return player != null && !player.IsDestroyed && player.userID == playerId &&
                player.IsConnected && player.IsAlive();
        }

        private static bool ValidIdentity(RiderIdentity value)
        { return value.PlayerId != 0 && value.RuntimeId != Guid.Empty && value.LifeId != Guid.Empty; }

        private static bool SameIdentity(RiderIdentity first, RiderIdentity second)
        {
            return first.PlayerId == second.PlayerId && first.RuntimeId == second.RuntimeId &&
                first.LifeId == second.LifeId;
        }
    }
}
