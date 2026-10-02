use serde::Serialize;
use std::collections::BTreeMap;

pub const MAX_FILE_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_BLOCK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapError(pub &'static str);
impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for MapError {}

#[derive(Clone, Debug, Serialize)]
pub struct Prefab {
    pub category: String,
    pub id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Clone, Debug, Serialize)]
pub struct Path {
    pub name: String,
    pub spline: bool,
    pub width: f32,
    pub nodes: Vec<[f32; 3]>,
}

#[derive(Clone, Debug)]
pub struct RustMap {
    pub size: u32,
    pub maps: BTreeMap<String, Vec<u8>>,
    pub prefabs: Vec<Prefab>,
    pub paths: Vec<Path>,
}

struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], MapError> {
        if n > self.0.len() {
            return Err(MapError("truncated map data"));
        }
        let (value, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(value)
    }
    fn varint(&mut self) -> Result<u64, MapError> {
        let mut value = 0;
        for shift in (0..70).step_by(7) {
            let byte = self.take(1)?[0];
            if shift == 63 && byte > 1 {
                return Err(MapError("varint overflow"));
            }
            value |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(MapError("varint overflow"))
    }
    fn fields(mut self) -> Result<Vec<(u32, Value<'a>)>, MapError> {
        let mut out = Vec::new();
        while !self.0.is_empty() {
            if out.len() >= 250_000 {
                return Err(MapError("too many protobuf fields"));
            }
            let key = self.varint()?;
            let tag = u32::try_from(key >> 3).map_err(|_| MapError("invalid protobuf tag"))?;
            if tag == 0 || tag > 0x1fff_ffff {
                return Err(MapError("invalid protobuf tag"));
            }
            let value = match key & 7 {
                0 => Value::Integer(self.varint()?),
                1 => {
                    self.take(8)?;
                    Value::Fixed64
                }
                2 => {
                    let n =
                        usize::try_from(self.varint()?).map_err(|_| MapError("length overflow"))?;
                    Value::Bytes(self.take(n)?)
                }
                5 => Value::Float(f32::from_le_bytes(self.take(4)?.try_into().unwrap())),
                _ => return Err(MapError("unsupported protobuf wire type")),
            };
            out.push((tag, value));
        }
        Ok(out)
    }
}

enum Value<'a> {
    Fixed64,
    Integer(u64),
    Bytes(&'a [u8]),
    Float(f32),
}
impl<'a> Value<'a> {
    fn integer(self) -> Result<u32, MapError> {
        match self {
            Self::Integer(v) => u32::try_from(v).map_err(|_| MapError("integer overflow")),
            _ => Err(MapError("expected integer")),
        }
    }
    fn bytes(self) -> Result<&'a [u8], MapError> {
        match self {
            Self::Bytes(b) => Ok(b),
            _ => Err(MapError("expected bytes")),
        }
    }
    fn float(self) -> Result<f32, MapError> {
        match self {
            Self::Float(v) if v.is_finite() => Ok(v),
            _ => Err(MapError("invalid float")),
        }
    }
    fn string(self) -> Result<String, MapError> {
        let b = self.bytes()?;
        if b.len() > 4096 {
            return Err(MapError("string too long"));
        }
        std::str::from_utf8(b)
            .map(str::to_owned)
            .map_err(|_| MapError("invalid UTF-8"))
    }
}

fn vector(bytes: &[u8], default: [f32; 3]) -> Result<[f32; 3], MapError> {
    let mut out = default;
    for (tag, value) in Cursor(bytes).fields()? {
        if (1..=3).contains(&tag) {
            out[(tag - 1) as usize] = value.float()?;
        }
    }
    Ok(out)
}

fn prefab(bytes: &[u8]) -> Result<Prefab, MapError> {
    let mut p = Prefab {
        category: String::new(),
        id: 0,
        position: [0.; 3],
        rotation: [0.; 3],
        scale: [1.; 3],
    };
    for (tag, value) in Cursor(bytes).fields()? {
        match tag {
            1 => p.category = value.string()?,
            2 => p.id = value.integer()?,
            3 => p.position = vector(value.bytes()?, [0.; 3])?,
            4 => p.rotation = vector(value.bytes()?, [0.; 3])?,
            5 => p.scale = vector(value.bytes()?, [1.; 3])?,
            _ => {}
        }
    }
    Ok(p)
}

fn path(bytes: &[u8]) -> Result<Path, MapError> {
    let mut p = Path {
        name: String::new(),
        spline: false,
        width: 0.,
        nodes: Vec::new(),
    };
    for (tag, value) in Cursor(bytes).fields()? {
        match tag {
            1 => p.name = value.string()?,
            2 => p.spline = value.integer()? != 0,
            5 => p.width = value.float()?,
            15 => {
                if p.nodes.len() >= 100_000 {
                    return Err(MapError("too many path nodes"));
                }
                p.nodes.push(vector(value.bytes()?, [0.; 3])?);
            }
            _ => {}
        }
    }
    Ok(p)
}

impl RustMap {
    pub fn read(file: &[u8]) -> Result<Self, MapError> {
        if file.len() > MAX_FILE_BYTES {
            return Err(MapError("map file exceeds limit"));
        }
        let mut input = Cursor(file);
        let version = u32::from_le_bytes(input.take(4)?.try_into().unwrap());
        if version != 9 {
            return Err(MapError(
                "only published Rust.World SDK map version 9 is supported",
            ));
        }
        let mut protobuf = Vec::new();
        while !input.0.is_empty() {
            let flags = input.varint()?;
            if flags & !3 != 0 {
                return Err(MapError("unsupported LZ4 stream flags"));
            }
            let size =
                usize::try_from(input.varint()?).map_err(|_| MapError("block length overflow"))?;
            if size > MAX_BLOCK_BYTES || size > MAX_FILE_BYTES - protobuf.len() {
                return Err(MapError("expanded map exceeds limit"));
            }
            if flags & 1 != 0 {
                let stored = usize::try_from(input.varint()?)
                    .map_err(|_| MapError("block length overflow"))?;
                if stored > size {
                    return Err(MapError("invalid compressed block length"));
                }
                let decoded = lz4_flex::block::decompress(input.take(stored)?, size)
                    .map_err(|_| MapError("invalid LZ4 block"))?;
                if decoded.len() != size {
                    return Err(MapError("expanded block length mismatch"));
                }
                protobuf.extend(decoded);
            } else {
                protobuf.extend(input.take(size)?);
            }
        }
        Self::read_protobuf(&protobuf)
    }
    pub fn read_protobuf(bytes: &[u8]) -> Result<Self, MapError> {
        if bytes.len() > MAX_FILE_BYTES {
            return Err(MapError("map protobuf exceeds limit"));
        }
        let mut world = Self {
            size: 0,
            maps: BTreeMap::new(),
            prefabs: Vec::new(),
            paths: Vec::new(),
        };
        for (tag, value) in Cursor(bytes).fields()? {
            match tag {
                1 => world.size = value.integer()?,
                2 => {
                    if world.maps.len() >= 64 {
                        return Err(MapError("too many terrain maps"));
                    }
                    let mut name = String::new();
                    let mut data = Vec::new();
                    for (tag, value) in Cursor(value.bytes()?).fields()? {
                        match tag {
                            1 => name = value.string()?,
                            2 => data = value.bytes()?.to_vec(),
                            _ => {}
                        }
                    }
                    if name.is_empty() || world.maps.insert(name, data).is_some() {
                        return Err(MapError("duplicate or unnamed terrain map"));
                    }
                }
                3 => {
                    if world.prefabs.len() >= 200_000 {
                        return Err(MapError("too many prefabs"));
                    }
                    world.prefabs.push(prefab(value.bytes()?)?);
                }
                4 => {
                    if world.paths.len() >= 10_000 {
                        return Err(MapError("too many paths"));
                    }
                    world.paths.push(path(value.bytes()?)?);
                }
                _ => {}
            }
        }
        if !(1..=20_000).contains(&world.size) {
            return Err(MapError("invalid world size"));
        }
        Ok(world)
    }
    pub fn heightfield(&self) -> Result<Heightfield, MapError> {
        let data = self
            .maps
            .get("terrain")
            .ok_or(MapError("terrain map is missing"))?;
        let count = data.len() / 2;
        let resolution = (count as f64).sqrt() as usize;
        if data.len() % 2 != 0 || resolution < 2 || resolution * resolution != count {
            return Err(MapError("terrain samples are not a square 16-bit grid"));
        }
        let heights = data
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32766. * 1000. - 500.)
            .collect();
        Ok(Heightfield {
            resolution,
            size: self.size as f32,
            heights,
        })
    }
}

pub struct Heightfield {
    pub resolution: usize,
    pub size: f32,
    pub heights: Vec<f32>,
}
impl Heightfield {
    pub fn sample(&self, x: f32, z: f32) -> Option<f32> {
        if self.resolution < 2
            || self.resolution.checked_mul(self.resolution) != Some(self.heights.len())
            || !self.size.is_finite()
            || self.size <= 0.
        {
            return None;
        }
        let half = self.size * 0.5;
        if !x.is_finite() || !z.is_finite() || x.abs() > half || z.abs() > half {
            return None;
        }
        let u = ((x + half) / self.size) * (self.resolution - 1) as f32;
        let v = ((z + half) / self.size) * (self.resolution - 1) as f32;
        let ix = (u.floor() as usize).min(self.resolution - 2);
        let iz = (v.floor() as usize).min(self.resolution - 2);
        let (a, b) = (u - ix as f32, v - iz as f32);
        let row = |r| {
            self.heights[r * self.resolution + ix] * (1. - a)
                + self.heights[r * self.resolution + ix + 1] * a
        };
        let height = row(iz) * (1. - b) + row(iz + 1) * b;
        height.is_finite().then_some(height)
    }
}
