# HorizonSpatialEngine

High-performance spatial compute engine for massive urban geometry datasets. Built in Rust with strict separation between the zero-copy compute core and the gRPC transport layer.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  horizon-server          (composition root / adapter)       │
├─────────────────────────────────────────────────────────────┤
│  horizon-transport       gRPC ──► horizon-api traits        │
│  horizon-proto           Protobuf wire types                │
├────────────────────────── boundary ─────────────────────────┤
│  horizon-api             transport-agnostic DTOs + traits   │
├─────────────────────────────────────────────────────────────┤
│  horizon-core            R-tree spatial queries             │
│  horizon_topology        PostGIS sqlx repository layer      │
│  horizon-storage         memmap + rkyv zero-copy access     │
│  horizon-geometry        archived urban geometry types      │
└─────────────────────────────────────────────────────────────┘
```

**Decoupling rule:** `horizon-transport` and `horizon-proto` never depend on `horizon-core`, `horizon-storage`, or `horizon-geometry`. The server binary is the only crate that wires both sides together via `CoreAdapter`.

## Workspace Crates

| Crate | Role |
|---|---|
| `horizon-geometry` | rkyv-archived `Building`, `Polygon`, `UrbanDataset` types |
| `horizon-storage` | Memory-mapped ingestion via `memmap2`, zero-copy `Archived*` access |
| `horizon-core` | Spatial index (R-tree), intersection & accessibility compute |
| `horizon_topology` | PostGIS `UrbanTopologyRepo` with compile-time verified sqlx queries |
| `horizon-api` | `SpatialService` trait + neutral request/response DTOs |
| `horizon-proto` | Protobuf schema + tonic code generation |
| `horizon-transport` | gRPC server; proto ↔ API conversion only |
| `horizon-server` | Binary entrypoint |

## Build

Requires Rust ≥ 1.78 (see `rust-toolchain.toml`).

```bash
cargo build --release
```

Release profile enables **LTO** and **codegen-units = 1** for maximum machine-code optimization:

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
```

## Run

```bash
HORIZON_LISTEN_ADDR=0.0.0.0:50051 cargo run -p horizon-server --release
```

## gRPC API

Service: `horizon.spatial.v1.SpatialService`

| RPC | Description |
|---|---|
| `LoadDataset` | Memory-map an rkyv urban geometry archive |
| `Intersect` | Bounding-box intersection against building footprints |
| `Accessibility` | Skyline obstruction & view-distance metrics |

## PostGIS Integration

The `horizon_topology` crate connects to PostGIS via **sqlx** with compile-time verified queries (`sqlx::query_as!`). Spatial pre-filtering happens entirely in the database using `ST_Intersects` + `ST_MakeEnvelope` before geometry enters Rust memory.

### Setup

```bash
cp .env.example .env
docker compose up -d
cargo install sqlx-cli --no-default-features --features postgres,rustls
cargo sqlx migrate run
./scripts/prepare_sqlx.ps1   # refresh .sqlx/ after query changes
```

### Coastal Neighborhoods

`UrbanTopologyRepo::fetch_coastal_neighborhoods()` loads 3D bounding boxes and polygon footprints for:

| Neighborhood | Region | Envelope (WGS84) |
|---|---|---|
| **Daryakenar** | Caspian coast, Mazandaran | 52.595–52.725°E, 36.675–36.745°N |
| **Iranshahr** | Gulf of Oman coast, Sistan-Baluchestan | 60.650–60.720°E, 27.180–27.240°N |

### Run with PostGIS

```bash
DATABASE_URL=postgres://horizon:horizon@localhost:5432/horizon_spatial \
  cargo run -p horizon-server --release
```

## Dataset Format

Urban geometry archives use rkyv zero-copy serialization. The root type is `UrbanDataset` (magic `HZRN`, version `1`). Write archives with `MmapLoader::serialize()` and load with `MmapLoader::load()` — geometry is accessed in-place from the mapped region with no redundant heap copies.

Enable byte-level validation at load time:

```bash
cargo build --release --features validation
```

## License

MIT OR Apache-2.0
