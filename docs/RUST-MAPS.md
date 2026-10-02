# Rust.World-mapinspectie

`rust_maps` leest de openbaar beschreven Facepunch/Rust.World **SDK-versie 9**:
een little-endian versienummer, de legacy LZ4.NET-chunkstream en protobufdata.
Een moderne Rust-servermap of andere mapversie is hiermee niet automatisch
ondersteund. Het bestandsformaat is geen assetpakket.

```bash
cargo run -p rust_maps --bin rust-map-inspect -- /pad/naar/sdk-v9.map
```

De JSON-output meldt wereldgrootte, terrainblobnamen/groottes, aantallen
prefabs en paden en, indien aanwezig, de hoogte in het midden.
De reader leest prefab-ID's, categorieën en transforms en padnamen,
splinevlaggen, breedtes en nodes. Overige padmetadata wordt nog niet gebruikt.

De `terrain`-blob wordt geïnterpreteerd als een vierkant signed-16-bit-grid
met de hoogteconversie van het publieke SDK. De hoogtequery interpoleert
bilineair binnen het gecentreerde wereldoppervlak.

Invoer wordt begrensd: 256 MiB bestand/uitgepakte data, 16 MiB per chunk,
64 terrainblobs, 200.000 prefabs en 10.000 paden. Ongeldige lengtes, LZ4,
protobuf, niet-eindige transforms en dubbele terrainnamen worden geweigerd.

Dit is **formatinspectie**, nog geen speelbare mapimport. Terrain-rendering,
collisie-installatie, oorspronkelijke prefabmeshes/materialen, water,
monumenten en koppeling met de MW2-wereld moeten nog worden gebouwd.
Er zijn geen originele Rust-gamebestanden of echte SDK-mapbestanden aanwezig.

Referentie: [Facepunch/Rust.World](https://github.com/Facepunch/Rust.World)
(MIT); zie de protobufdeclaraties, WorldSerialization, TerrainMap en BitUtility.
