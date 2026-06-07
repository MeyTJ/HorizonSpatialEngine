//! Minimal EWKB polygon ring decoder for PostGIS `ST_AsEWKB` output.

use crate::TopologyError;

const WKB_POLYGON: u32 = 3;
const WKB_MULTI_POLYGON: u32 = 5;
const WKB_SRID_FLAG: u32 = 0x2000_0000;

pub fn decode_ewkb_footprint(bytes: &[u8]) -> Result<Vec<(f64, f64)>, TopologyError> {
    if bytes.len() < 5 {
        return Err(TopologyError::WkbDecode("buffer too short".into()));
    }

    let little_endian = bytes[0] == 1;
    let mut offset = 1usize;
    let raw_type = read_u32(bytes, &mut offset, little_endian)?;
    let base_type = raw_type & !WKB_SRID_FLAG;

    if raw_type & WKB_SRID_FLAG != 0 {
        let _srid = read_u32(bytes, &mut offset, little_endian)?;
    }

    match base_type {
        WKB_POLYGON => read_polygon_ring(bytes, &mut offset, little_endian),
        WKB_MULTI_POLYGON => {
            let polygon_count = read_u32(bytes, &mut offset, little_endian)?;
            if polygon_count == 0 {
                return Err(TopologyError::EmptyRing);
            }
            let inner_little = bytes[offset] == 1;
            offset += 1;
            let inner_type = read_u32(bytes, &mut offset, inner_little)?;
            let inner_base = inner_type & !WKB_SRID_FLAG;
            if inner_base != WKB_POLYGON {
                return Err(TopologyError::WkbDecode(format!(
                    "unsupported inner geometry type {inner_base}"
                )));
            }
            if inner_type & WKB_SRID_FLAG != 0 {
                let _srid = read_u32(bytes, &mut offset, inner_little)?;
            }
            read_polygon_ring(bytes, &mut offset, inner_little)
        }
        other => Err(TopologyError::WkbDecode(format!(
            "unsupported geometry type {other}"
        ))),
    }
}

fn read_polygon_ring(
    bytes: &[u8],
    offset: &mut usize,
    little_endian: bool,
) -> Result<Vec<(f64, f64)>, TopologyError> {
    let ring_count = read_u32(bytes, offset, little_endian)?;
    if ring_count == 0 {
        return Err(TopologyError::EmptyRing);
    }

    let point_count = read_u32(bytes, offset, little_endian)?;
    if point_count < 3 {
        return Err(TopologyError::EmptyRing);
    }

    let mut ring = Vec::with_capacity(point_count as usize);
    for _ in 0..point_count {
        let x = read_f64(bytes, offset, little_endian)?;
        let y = read_f64(bytes, offset, little_endian)?;
        ring.push((x, y));
    }

    Ok(ring)
}

fn read_u32(bytes: &[u8], offset: &mut usize, little_endian: bool) -> Result<u32, TopologyError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| TopologyError::WkbDecode("offset overflow".into()))?;
    if end > bytes.len() {
        return Err(TopologyError::WkbDecode("unexpected end of WKB".into()));
    }
    let value = if little_endian {
        u32::from_le_bytes(bytes[*offset..end].try_into().map_err(|_| {
            TopologyError::WkbDecode("invalid u32 slice".into())
        })?)
    } else {
        u32::from_be_bytes(bytes[*offset..end].try_into().map_err(|_| {
            TopologyError::WkbDecode("invalid u32 slice".into())
        })?)
    };
    *offset = end;
    Ok(value)
}

fn read_f64(bytes: &[u8], offset: &mut usize, little_endian: bool) -> Result<f64, TopologyError> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| TopologyError::WkbDecode("offset overflow".into()))?;
    if end > bytes.len() {
        return Err(TopologyError::WkbDecode("unexpected end of WKB".into()));
    }
    let value = if little_endian {
        f64::from_le_bytes(bytes[*offset..end].try_into().map_err(|_| {
            TopologyError::WkbDecode("invalid f64 slice".into())
        })?)
    } else {
        f64::from_be_bytes(bytes[*offset..end].try_into().map_err(|_| {
            TopologyError::WkbDecode("invalid f64 slice".into())
        })?)
    };
    *offset = end;
    Ok(value)
}

pub fn to_horizon_polygon(
    ring: &[(f64, f64)],
    elevation_min: f64,
    elevation_max: f64,
) -> horizon_geometry::Polygon {
    let vertices = ring
        .iter()
        .map(|&(x, y)| horizon_geometry::Point3::new(x, y, elevation_min))
        .collect();

    horizon_geometry::Polygon::new(vertices, elevation_min, elevation_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_little_endian_ewkb_polygon() {
        let mut wkb = Vec::new();
        wkb.push(1);
        wkb.extend_from_slice(&(WKB_POLYGON | WKB_SRID_FLAG).to_le_bytes());
        wkb.extend_from_slice(&4326u32.to_le_bytes());
        wkb.extend_from_slice(&1u32.to_le_bytes());
        wkb.extend_from_slice(&4u32.to_le_bytes());
        wkb.extend_from_slice(&52.60f64.to_le_bytes());
        wkb.extend_from_slice(&36.68f64.to_le_bytes());
        wkb.extend_from_slice(&52.62f64.to_le_bytes());
        wkb.extend_from_slice(&36.68f64.to_le_bytes());
        wkb.extend_from_slice(&52.62f64.to_le_bytes());
        wkb.extend_from_slice(&36.70f64.to_le_bytes());
        wkb.extend_from_slice(&52.60f64.to_le_bytes());
        wkb.extend_from_slice(&36.70f64.to_le_bytes());
        wkb.extend_from_slice(&52.60f64.to_le_bytes());
        wkb.extend_from_slice(&36.68f64.to_le_bytes());

        let ring = decode_ewkb_footprint(&wkb).expect("valid ewkb");
        assert_eq!(ring.len(), 4);
    }
}
