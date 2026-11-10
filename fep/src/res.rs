//! Resources an FEP names but does not hold: `effect/resident/bin/resident.frr` (README 9.2),
//! and the lookup of any resource's bytes (README 9.1).
//!
//! resident.frr: `"FRR\0"`, `+0x0c` table count (4), `+0x14` counts (32 TEX, 134 ANM, 136 MDL,
//! 24 VTL), then 4 offsets of runtime-slot tables (-1s, INFERRED) and 4 offsets of tables of
//! resource file offsets, in the order TEX, ANM, MDL, VTL. Each resource is the same block as in
//! an FEP. A resident id is `0xF0000000 | (k >> 7) << 26 | type << 7 | (k & 0x7f)` for entry `k`
//! of table `type` (326/326 ids used game-wide exist). The frr holds no sizes: a resource here
//! runs to the next one's start (or the file end). The code that registers these with owner 0
//! was not found; DDD finds them by id (`0x55ae10`).

use crate::parse::{Bytes, Error, Fep, ResKind, Resource, Result};

#[derive(Debug, Clone)]
pub struct Resident {
    pub data: Vec<u8>,
    /// File offsets per table: TEX, ANM, MDL, VTL.
    pub tables: [Vec<usize>; 4],
}

impl Resident {
    pub fn parse(data: Vec<u8>) -> Result<Resident> {
        let b = Bytes(&data);
        if b.slice(0, 4, "frr magic")? != b"FRR\0" {
            return Err(Error::Frr("not FRR"));
        }
        let n = b.u32(0x0c, "frr")? as usize;
        if n != 4 {
            return Err(Error::Frr("not 4 tables"));
        }
        let mut tables: [Vec<usize>; 4] = Default::default();
        for (t, table) in tables.iter_mut().enumerate() {
            let count = b.u32(0x14 + 4 * t, "frr counts")? as usize;
            let at = b.u32(0x14 + 4 * n + 4 * (n + t), "frr tables")? as usize;
            *table = (0..count).map(|k| Ok(b.u32(at + 4 * k, "frr table")? as usize)).collect::<Result<_>>()?;
        }
        Ok(Resident { data, tables })
    }

    /// Entry `k` of a table's bytes, up to the next resource of any table or the file end.
    pub fn by_index(&self, kind: ResKind, k: usize) -> Option<&[u8]> {
        let at = *self.tables[kind as usize].get(k)?;
        let end = self.tables.iter().flatten().copied().filter(|&o| o > at).min().unwrap_or(self.data.len());
        self.data.get(at..end.min(self.data.len()))
    }

    /// A resident id's table and entry, if it is one (README 9.1).
    pub fn decode(id: u32) -> Option<(ResKind, usize)> {
        if id & 0x7800_0000 != 0x7000_0000 {
            return None;
        }
        let kind = match (id >> 7) & 3 {
            0 => ResKind::Tex,
            1 => ResKind::Anm,
            2 => ResKind::Mdl,
            _ => ResKind::Vtl,
        };
        Some((kind, (id & 0x7f) as usize | (((id >> 26) & 1) as usize) << 7))
    }

    /// A resident resource's bytes by id.
    pub fn get(&self, id: u32) -> Option<&[u8]> {
        let (kind, k) = Self::decode(id)?;
        self.by_index(kind, k)
    }
}

/// A resource's bytes: local ones from the FEP, resident ones from resident.frr (when given).
/// `None` for an id-0 record, which DDD skips (`0x5b3076`), and for a resident one without the frr.
pub fn data<'a>(fep: &'a Fep, resident: Option<&'a Resident>, r: &Resource) -> Option<&'a [u8]> {
    if r.id == 0 {
        None
    } else if r.is_resident() {
        resident?.get(r.id)
    } else {
        fep.local_data(r)
    }
}
