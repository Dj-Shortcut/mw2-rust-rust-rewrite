use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: rust-map-inspect <mapfile>")?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((rust_maps::MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let map = rust_maps::RustMap::read(&bytes)?;
    let blobs: Vec<_> = map
        .maps
        .iter()
        .map(|(name, data)| serde_json::json!({"name":name,"bytes":data.len()}))
        .collect();
    let height = map.heightfield().ok().and_then(|h| h.sample(0., 0.));
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"map_version":9,"size_metres":map.size,"maps":blobs,"prefabs":map.prefabs.len(),"paths":map.paths.len(),"center_height_metres":height,"status":"format inspection only; prefab assets and runtime terrain installation are not implemented"})
        )?
    );
    Ok(())
}
