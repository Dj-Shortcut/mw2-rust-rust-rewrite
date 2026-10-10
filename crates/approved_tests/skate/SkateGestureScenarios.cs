using System;
using System.Globalization;
using Shortcut.RustMod;

internal static class SkateGestureScenarios
{
    private static int Main()
    {
        try
        {
            Replay("skate_air_pullback_ollie", "0:-9,-29 21:-10,-60 51:-15,-87 65:-22,-93 81:-24,-95 106:-25,-96 127:-26,-96 131:-26,-96 144:-26,-96 156:-27,-96 169:-27,-96 182:-27,-97 195:-27,-97 207:-27,-97 229:-27,-98 242:-26,-98 251:-22,-100 259:-16,-100 285:7,-100 302:12,-100 315:36,-100 324:39,-100 337:42,-99 361:46,-98 374:52,-96 388:58,-73 400:62,8 439:62,100 455:38,100 468:15,100 472:7,100 498:0,100 506:-6,100 524:-10,99 528:-13,62 545:-15,32 576:-16,26 601:-17,3 606:-17,1 611:-17,-1 628:-17,-3 654:-17,-5 671:-17,-8", true, SkateFlickEvents.Ollie, 0, false);
            Replay("skate_ground_diagonal_kickflip", "0:20,-28 16:20,-52 38:20,-85 50:20,-90 63:20,-93 76:20,-96 97:20,-97 109:20,-98 117:20,-98 130:20,-99 160:20,-99 174:20,-99 221:20,-99 1021:20,-99 1268:20,-100 1461:20,-100 1488:20,-100 1521:20,-100 1530:20,-100 1534:20,-100 1546:15,-100 1555:-7,-32 1584:-35,80 1597:-39,86 1609:-42,90 1622:-44,92 1630:-45,93 1655:-46,94 1672:-46,95 1681:-46,95 1694:-47,95 1736:-47,95 1740:-47,96 1744:-47,96 1761:-47,96 1769:-47,96 1790:-47,96 1816:-47,95 1832:-47,55 1841:2,11 1853:2,7 1865:2,5 1892:2,3 1904:2,3 1913:2,2 1926:2,2", false, SkateFlickEvents.Ollie | SkateFlickEvents.Kickflip, 1, false);
            Replay("skate_ground_roll_ollie_no_manual", "0:-43,-37 4:-70,-41 16:-77,-46 41:-81,-50 53:-84,-52 66:-86,-53 87:-88,-54 96:-88,-56 125:-89,-64 138:-88,-70 155:-65,-77 173:-59,-83 189:-32,-100 202:-7,-100 210:-2,-100 228:22,-100 253:25,-100 263:28,-100 275:31,-100 284:35,-100 309:38,-100 326:41,-38 343:40,62 355:12,100 372:6,100 381:-2,100 412:-12,100 433:-15,74 449:-17,50 466:-18,18 483:-19,-9 513:-20,-14 526:-20,-20 543:-20,-28 573:-20,-30 590:-20,-32 604:-20,-34 616:-20,-33 637:-20,-31 650:-19,-28 662:-18,-26 675:-17,-25 697:-15,-22 709:-14,-19 744:-12,-17 762:-10,-13 770:-10,-12 788:-9,-11 797:-9,-10 819:-9,-10 832:-9,-9 849:-8,-9", false, SkateFlickEvents.Ollie, 0, true);
            Console.WriteLine("PASS: all three owner-recorded skate gesture scenarios.");
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine(error.Message);
            return 1;
        }
    }

    private static void Replay(string name, string trace, bool airborne, SkateFlickEvents expected, double flips, bool noManual)
    {
        string[] samples = trace.Split(' ');
        var state = default(SkateFlickState);
        int pops = 0;
        double previousTime = 0, previousX = 0, previousY = 0;
        for (int i = 0; i < samples.Length; i++)
        {
            string[] timeAndPosition = samples[i].Split(':');
            string[] axes = timeAndPosition[1].Split(',');
            double time = double.Parse(timeAndPosition[0], CultureInfo.InvariantCulture) / 1000;
            double x = double.Parse(axes[0], CultureInfo.InvariantCulture) / 100;
            double y = double.Parse(axes[1], CultureInfo.InvariantCulture) / 100;
            if (i != 0)
                Hold(name, ref state, previousX, previousY, airborne, time - previousTime, expected, flips, noManual, ref pops);
            previousTime = time; previousX = x; previousY = y;
        }
        Hold(name, ref state, previousX, previousY, airborne, 0.05, expected, flips, noManual, ref pops);
        Require(pops == 1, name + ": expected exactly one composed ollie, got " + pops + ".");
        Console.WriteLine("PASS: " + name + " (" + samples.Length + " recorded positions).");
    }

    private static void Hold(string name, ref SkateFlickState state, double x, double y, bool air, double seconds,
                             SkateFlickEvents expected, double flips, bool noManual, ref int pops)
    {
        int count = (int)Math.Ceiling(seconds / SkateFlick.MaximumStep);
        Require(count > 0, name + ": nonpositive recorded interval.");
        double dt = seconds / count;
        for (int i = 0; i < count; i++)
        {
            SkateFlickResult result; string error;
            Require(SkateFlick.TryStep(state, new SkateFlickInput(x, y, air, false), dt, out result, out error),
                    name + ": rejected recorded position: " + error);
            state = result.State;
            if (noManual) Require(!result.Manual && !result.NoseManual, name + ": stick recoil became a manual.");
            if (result.Events == SkateFlickEvents.None) continue;
            pops++;
            Require(result.Events == expected, name + ": wrong event " + result.Events + ".");
            Require(result.Composition.Kind == SkatePopKind.Ollie, name + ": missing composed ollie.");
            Require(result.Composition.FlipTurns == flips, name + ": wrong flip turns.");
            Require(result.Composition.ShoveDegrees == 0, name + ": unexpected shove.");
            Require(result.Pop > 0 && result.Pop <= 1 && result.Composition.Strength == result.Pop,
                    name + ": invalid or inconsistent pop strength.");
        }
    }

    private static void Require(bool condition, string message)
    {
        if (!condition) throw new InvalidOperationException(message);
    }
}
