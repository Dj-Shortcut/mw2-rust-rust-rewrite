// Offline interop generation for the currently installed Rust build (issue #289).
// Mirrors BepInEx be.788 Il2CppInteropManager, but uses the patched Cpp2IL BuildInteropSourceModels
// entry point and the patched Il2CppInterop generator already staged on Shadow. Reads the game
// files only; writes only to the given fresh output directory.
using System;
using System.Collections.Generic;
using System.IO;
using AssetRipper.Primitives;
using Cpp2IL.Core;
using Cpp2IL.Core.Api;
using Cpp2IL.Core.InstructionSets;
using Cpp2IL.Core.OutputFormats;
using Cpp2IL.Core.ProcessingLayers;
using Il2CppInterop.Common;
using Il2CppInterop.Generator;
using Il2CppInterop.Generator.Runners;
using LibCpp2IL;
using Microsoft.Extensions.Logging;

public static class Gen
{
    private static void Say(string m) { Console.WriteLine(DateTime.Now.ToString("HH:mm:ss") + " " + m); Console.Out.Flush(); }

    public static int Main(string[] a)
    {
        try
        {
            string ga = a[0], meta = a[1], unityLibs = a[2], outDir = a[3];
            if (Directory.Exists(outDir) && Directory.GetFileSystemEntries(outDir).Length > 0) { Say("FAIL output directory is not empty"); return 2; }
            InstructionSetRegistry.RegisterInstructionSet<X86InstructionSet>(DefaultInstructionSets.X86_32);
            InstructionSetRegistry.RegisterInstructionSet<X86InstructionSet>(DefaultInstructionSets.X86_64);
            LibCpp2IlBinaryRegistry.RegisterBuiltInBinarySupport();
            Cpp2IL.Core.Logging.Logger.InfoLog += (m, s) => Say("[cpp2il " + s + "] " + m.Trim());
            Cpp2IL.Core.Logging.Logger.WarningLog += (m, s) => Say("[cpp2il WARN " + s + "] " + m.Trim());
            Cpp2IL.Core.Logging.Logger.ErrorLog += (m, s) => Say("[cpp2il ERROR " + s + "] " + m.Trim());

            Say("STAGE init");
            Cpp2IlApi.InitializeLibCpp2Il(ga, meta, new UnityVersion(6000, 3, 15), false);
            var layers = new List<Cpp2IlProcessingLayer> { new AttributeInjectorProcessingLayer() };
            foreach (var l in layers) l.PreProcess(Cpp2IlApi.CurrentAppContext, layers);
            foreach (var l in layers) l.Process(Cpp2IlApi.CurrentAppContext);
            Say("STAGE source models");
            var asms = new AsmResolverDllOutputFormatDefault().BuildInteropSourceModels(Cpp2IlApi.CurrentAppContext);
            Say("source assemblies=" + asms.Count);
            LibCpp2IlMain.Reset();
            Cpp2IlApi.CurrentAppContext = null;

            Say("STAGE interop generation");
            Directory.CreateDirectory(outDir);
            var opts = new GeneratorOptions
            {
                GameAssemblyPath = ga,
                Source = asms,
                OutputDir = outDir,
                UnityBaseLibsDir = Directory.Exists(unityLibs) ? unityLibs : null,
            };
            var log = new L();
            Il2CppInteropGenerator.Create(opts).AddLogger(log).AddInteropAssemblyGenerator().Run();
            Say("GEN DONE files=" + Directory.GetFiles(outDir).Length + " warnings=" + log.Warn + " errors=" + log.Err);
            return 0;
        }
        catch (Exception e)
        {
            Say("GEN FAIL " + e);
            return 1;
        }
    }

    private sealed class L : ILogger
    {
        public int Warn, Err;
        public IDisposable BeginScope<TState>(TState state) { return null; }
        public bool IsEnabled(LogLevel level) { return level >= LogLevel.Information; }
        public void Log<TState>(LogLevel level, EventId id, TState state, Exception ex, Func<TState, Exception, string> f)
        {
            if (level < LogLevel.Information) return;
            if (level == LogLevel.Warning) Warn++;
            if (level >= LogLevel.Error) Err++;
            if (level == LogLevel.Warning && Warn > 40) return;
            Say("[interop " + level + "] " + f(state, ex));
        }
    }
}
