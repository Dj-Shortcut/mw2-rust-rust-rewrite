using System;

namespace Shortcut.RustMod
{
    public enum SkateSound { Rolling, Grind, Push, Ollie, Landing, Bail }

    public static class SkateAudio
    {
        public const int MinimumSampleRate = 8000;
        public const int MaximumSampleRate = 192000;
        public const double MinimumDuration = 0.02;
        public const double MaximumDuration = 10;

        public static bool TryCreate(SkateSound sound, int sampleRate, uint seed,
                                     out float[] samples, out string error)
        {
            double duration;
            switch (sound)
            {
                case SkateSound.Rolling: case SkateSound.Grind: duration = 1; break;
                case SkateSound.Push: duration = 0.18; break;
                case SkateSound.Ollie: duration = 0.14; break;
                case SkateSound.Landing: duration = 0.28; break;
                case SkateSound.Bail: duration = 0.65; break;
                default: samples = null; error = "Invalid skate sound request."; return false;
            }
            return TryCreate(sound, sampleRate, duration, seed, out samples, out error);
        }

        public static bool TryCreate(SkateSound sound, int sampleRate, double durationSeconds, uint seed,
                                     out float[] samples, out string error)
        {
            samples = null;
            error = null;
            if ((int)sound < 0 || (int)sound > (int)SkateSound.Bail || sampleRate < MinimumSampleRate ||
                sampleRate > MaximumSampleRate || double.IsNaN(durationSeconds) || double.IsInfinity(durationSeconds) ||
                durationSeconds < MinimumDuration || durationSeconds > MaximumDuration)
            { error = "Invalid skate sound request."; return false; }
            try
            {
                var candidate = new float[(int)Math.Round(sampleRate * durationSeconds, MidpointRounding.AwayFromZero)];
                uint random = unchecked(seed ^ (0x9e3779b9u * ((uint)sound + 1)));
                if (random == 0) random = 0x6d2b79f5u;
                if (sound == SkateSound.Rolling || sound == SkateSound.Grind) Loop(sound, sampleRate, candidate, ref random);
                else Shot(sound, sampleRate, candidate, ref random);
                samples = candidate;
                return true;
            }
            catch (OutOfMemoryException)
            { error = "Not enough memory to create skate sound."; return false; }
        }

        private static void Loop(SkateSound sound, int rate, float[] samples, ref uint random)
        {
            const int partials = 64;
            var bins = new int[partials];
            var weights = new double[partials];
            bool grind = sound == SkateSound.Grind;
            int low = Math.Max(1, (int)Math.Ceiling((grind ? 120 : 20) * samples.Length / (double)rate));
            int high = Math.Min((samples.Length - 1) / 2,
                Math.Max(low, (int)Math.Floor(Math.Min(grind ? 7000 : 2500, rate * 0.45) * samples.Length / rate)));
            low = Math.Min(low, high);
            for (int k = 0; k < partials; k++)
            {
                bins[k] = low + (int)(Next(ref random) % (uint)(high - low + 1));
                weights[k] = Noise(ref random) / Math.Pow(bins[k] / (double)low, grind ? 0.2 : 0.7);
            }
            int wheel = Math.Max(1, Math.Min(high, (int)Math.Round(35 * samples.Length / (double)rate)));
            for (int i = 1; i <= (samples.Length - 1) / 2; i++)
            {
                double phase = 2 * Math.PI * i / samples.Length;
                double value = grind ? 0 : 0.6 * Math.Sin(phase * wheel);
                for (int k = 0; k < partials; k++) value += weights[k] * Math.Sin(phase * bins[k]);
                samples[i] = (float)value;
                samples[samples.Length - i] = -samples[i];
            }
            Normalise(samples, grind ? 0.65 : 0.45);
        }

        private static void Shot(SkateSound sound, int rate, float[] samples, ref uint random)
        {
            double frequency, noise, decay;
            switch (sound)
            {
                case SkateSound.Push: frequency = 95; noise = 0.75; decay = 3; break;
                case SkateSound.Ollie: frequency = 220; noise = 0.55; decay = 8; break;
                case SkateSound.Landing: frequency = 75; noise = 0.5; decay = 5; break;
                default: frequency = 55; noise = 0.9; decay = 2.5; break;
            }
            frequency *= 0.9 + 0.2 * ((Noise(ref random) + 1) * 0.5);
            double phase = 0, previousNoise = 0, sum = 0, weightSum = 0;
            for (int i = 1; i < samples.Length - 1; i++)
            {
                double progress = i / (double)(samples.Length - 1);
                double envelope = Math.Sin(Math.PI * progress) * Math.Exp(-decay * progress);
                double currentNoise = Noise(ref random);
                previousNoise = previousNoise * 0.4 + currentNoise * 0.6;
                phase += 2 * Math.PI * frequency * (1 - 0.65 * progress) / rate;
                double value = envelope * ((1 - noise) * Math.Sin(phase) + noise * previousNoise);
                if (sound == SkateSound.Bail)
                    value += 0.2 * envelope * Math.Sin(progress * Math.PI * 12) * previousNoise;
                samples[i] = (float)value;
                sum += samples[i];
                weightSum += Math.Sin(Math.PI * progress);
            }
            double correction = sum / weightSum;
            for (int i = 1; i < samples.Length - 1; i++)
                samples[i] = (float)(samples[i] - correction * Math.Sin(Math.PI * i / (samples.Length - 1)));
            Normalise(samples, 0.8);
        }

        private static void Normalise(float[] samples, double peak)
        {
            double maximum = 0;
            for (int i = 0; i < samples.Length; i++) maximum = Math.Max(maximum, Math.Abs(samples[i]));
            if (maximum == 0) return;
            double gain = peak / maximum;
            for (int i = 0; i < samples.Length; i++) samples[i] = (float)(samples[i] * gain);
        }
        private static uint Next(ref uint state)
        { state ^= state << 13; state ^= state >> 17; state ^= state << 5; return state; }
        private static double Noise(ref uint state) { return (Next(ref state) >> 8) / 8388608.0 - 1; }
    }
}
