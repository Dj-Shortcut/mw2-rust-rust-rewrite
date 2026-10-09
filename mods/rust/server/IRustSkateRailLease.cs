namespace Shortcut.RustMod
{
    public interface IRustSkateRailLease
    {
        SkateRailBinding Binding { get; }
        bool IsCurrent { get; }
    }
}
