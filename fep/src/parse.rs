//! The FEP container, read straight from the file (docs/fep/README.md sections 1-6 and 9).
//!
//! - Offsets: in the file every 32-bit offset counts from the file start. DDD's loader (`0x5ab330`
//!   -> `0x5aadd0`) rebases the ones under section 1 to be relative to the struct holding them;
//!   we never do, so every offset here is file-relative. The u16 parameter offsets of command
//!   entries count from their list header (never rebased).
//! - Zero offsets: DDD treats a zero 32-bit offset as "none". Where it then still reads a count's
//!   worth of u16 (spawn refs, slot inputs) it reads 0 for each (`0x5b4648`, `0x58261e`); we do
//!   the same.
//! - Every read is bounds-checked: a bad file is an `Err`, never a panic.

use std::fmt;

use crate::sig;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A read past the end of the data: what was being read, at which file offset.
    Truncated(&'static str, usize),
    /// Not "FEP" (`0x5ab93a..0x5ab953`).
    NotFep,
    /// Header `+0x04` / `+0x06` not 1 / 9 (`0x5ab959`, `0x5ab965`).
    Version(i16, i16),
    /// The command signature table differs from the exe's (`0x9e7d10`); the first differing id.
    Signature(usize),
    /// A slot names a command id past the table (DDD refuses the element, `0x5ac2e8`).
    BadCommand(u16),
    /// A slot index past its part's slot count (in an entry, an input or a slot list).
    BadSlot { part_offset: usize, slot: u16 },
    /// No element by that name (`0x5abb70`) or id (`0x5acb40`).
    NoElement(String),
    /// Element id with bit 9 set: a "special" element (section 6), never present, not decoded.
    Special(u32),
    /// Start frame at or past the body's length (`0x5ac30c`).
    StartPastEnd { start: f32, length: i32 },
    /// resident.frr: what was wrong.
    Frr(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Truncated(what, at) => write!(f, "truncated: {what} at {at:#x}"),
            Error::NotFep => write!(f, "not an FEP file"),
            Error::Version(a, b) => write!(f, "FEP version {a}.{b}, want 1.9"),
            Error::Signature(id) => write!(f, "command signature table differs from DDD's at id {id}"),
            Error::BadCommand(id) => write!(f, "command id {id} past the table"),
            Error::BadSlot { part_offset, slot } => write!(f, "part at {part_offset:#x}: slot {slot} past its slots"),
            Error::NoElement(n) => write!(f, "no element {n}"),
            Error::Special(id) => write!(f, "element {id:#x} is a special element (not decoded)"),
            Error::StartPastEnd { start, length } => write!(f, "start frame {start} at or past the length {length}"),
            Error::Frr(what) => write!(f, "resident.frr: {what}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Bounds-checked little-endian reads.
#[derive(Clone, Copy)]
pub(crate) struct Bytes<'a>(pub &'a [u8]);

impl<'a> Bytes<'a> {
    pub fn slice(&self, o: usize, n: usize, what: &'static str) -> Result<&'a [u8]> {
        self.0.get(o..o.checked_add(n).ok_or(Error::Truncated(what, o))?).ok_or(Error::Truncated(what, o))
    }
    pub fn u8(&self, o: usize, what: &'static str) -> Result<u8> {
        self.0.get(o).copied().ok_or(Error::Truncated(what, o))
    }
    pub fn u16(&self, o: usize, what: &'static str) -> Result<u16> {
        let b = self.slice(o, 2, what)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn i16(&self, o: usize, what: &'static str) -> Result<i16> {
        Ok(self.u16(o, what)? as i16)
    }
    pub fn u32(&self, o: usize, what: &'static str) -> Result<u32> {
        let b = self.slice(o, 4, what)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn i32(&self, o: usize, what: &'static str) -> Result<i32> {
        Ok(self.u32(o, what)? as i32)
    }
    pub fn f32(&self, o: usize, what: &'static str) -> Result<f32> {
        Ok(f32::from_bits(self.u32(o, what)?))
    }
    fn vec4(&self, o: usize, what: &'static str) -> Result<[f32; 4]> {
        Ok([self.f32(o, what)?, self.f32(o + 4, what)?, self.f32(o + 8, what)?, self.f32(o + 12, what)?])
    }
    /// A table of `n` u32 file offsets at `o`.
    fn offsets(&self, o: usize, n: usize, what: &'static str) -> Result<Vec<usize>> {
        (0..n).map(|k| Ok(self.u32(o + 4 * k, what)? as usize)).collect()
    }
    /// `n` u16 at `o`; all 0 when `o` is 0 (DDD reads index 0 then).
    fn u16s(&self, o: usize, n: usize, what: &'static str) -> Result<Vec<u16>> {
        if o == 0 {
            return Ok(vec![0; n]);
        }
        (0..n).map(|k| self.u16(o + 2 * k, what)).collect()
    }
}

/// Header (0x48 bytes, README section 2).
#[derive(Debug, Clone)]
pub struct Header {
    /// Byte 3 after "FEP": not checked (0..65 over the game; GUESS a save counter).
    pub byte3: u8,
    /// `+0x10`: every element id is `id_base | group << 10 | element` (`0x5aadc0`).
    pub id_base: u32,
    /// `+0x14`.
    pub file_size: u32,
    /// `+0x18`, masked to 4 bits at `0x5ab971`; = id base >> 28 in every file.
    pub category: u16,
    /// `+0x1c..0x26`: counts of sections 0 (groups), 1 (bodies), 2 (TEX), 3 (MDL), 4 (ANM), 5 (VTL).
    pub counts: [u16; 6],
    /// `+0x28..0x44`: offsets of sections 0..7.
    pub sections: [u32; 8],
}

/// Name record (section 7): `u32 element id, u32 length, u32 -> name` (`0x5abb70`).
#[derive(Debug, Clone)]
pub struct Name {
    pub id: u32,
    pub name: String,
}

/// Group record (section 0): `u32 id, u16 body, u16 count, u32 -> u32[count] element offsets`.
#[derive(Debug, Clone)]
pub struct Group {
    pub id: u32,
    /// Body index (section 1) all its elements use (`0x5acd39`).
    pub body: u16,
    pub elements: Vec<Element>,
}

/// Element (0x9c bytes; an instance points to it at `+0x1a8`). Fields README section 4.
#[derive(Debug, Clone)]
pub struct Element {
    pub offset: usize,
    pub id: u32,
    /// `+0x06` view-angle test mode (`0x5b4a0b`), `+0x0a` view-bounds test (`0x5b4903`),
    /// `+0x0b` distance test (`0x5b49c2`). Not run here: they gate DDD's pass-1 update.
    pub view_angle: u8,
    pub view_bounds: u8,
    pub distance: u8,
    /// `+0x07` -> instance `+0x1bc`; `+0x8e` -> instance `+0x1be`. Meaning unknown.
    pub b07: u8,
    pub b8e: u8,
    /// `+0x10`: added to the start position (`0x5ac67b..0x5ac6ce`).
    pub start_offset: [f32; 4],
    /// `+0x30`: no reader found (GUESS scale).
    pub v30: [f32; 4],
    /// `+0x90`: start delay; the instance clock starts at minus this (`0x5ac4eb..0x5ac501`).
    pub start_delay: u16,
}

/// Body (section 1, 0x34 bytes, README section 5).
#[derive(Debug, Clone)]
pub struct Body {
    pub offset: usize,
    /// `+0x00` -> instance `+0x04`: bit 1 looping effect, bit 0 ends when no parts are left.
    pub flags: u16,
    /// `+0x04` length in frames -> instance `+0x16c` (`0x5ac530`).
    pub length: i32,
    /// `+0x0a` -> instance `+0x1b7` (`0x5ac449`).
    pub kind: u8,
    /// `+0x0c`: what a stop request does (`0x5b43a7`): 0 fade 30 frames, 1 fade 15, 3 ignore,
    /// other remove at once.
    pub stop_mode: u32,
    /// Not understood: `+0x02`, `+0x08`, `+0x0b`.
    pub u02: u16,
    pub u08: u16,
    pub b0b: u8,
    /// `+0x1c`: spawn events; `None` = a zero entry.
    pub events: Vec<Option<SpawnEvent>>,
    /// `+0x20`: parts; `None` = a zero entry (`0x5b7428`).
    pub parts: Vec<Option<Part>>,
    /// `+0x24`: every command id its parts use (checked at creation, `0x5ac2c2`).
    pub command_ids: Vec<u16>,
    /// `+0x28`: 12 u16 lists, relocated by `0x5ab0b2` but read by nothing found.
    pub lists28: Vec<Vec<u16>>,
    /// `+0x2c`: part groups, `{s16, u16 ref}` records (`0x5b0e80`, `0x5b0ec0`). INFERRED unused
    /// (README section 5).
    pub part_groups: Vec<Vec<(i16, u16)>>,
    /// `+0x1a`: count of `+0x30` records (0 in every file; not decoded).
    pub n30: u16,
}

/// Spawn event: `s16 frame, s16 n, s32 -> u16 part ref[n]` (`0x5b45b2..0x5b4693`).
#[derive(Debug, Clone)]
pub struct SpawnEvent {
    pub frame: i16,
    pub refs: Vec<u16>,
}

/// Part (0x48 bytes, README 6.1).
#[derive(Debug, Clone)]
pub struct Part {
    pub offset: usize,
    /// `+0x00`: bit 0 lives on after its life while it has children (`0x5b4542`); bit 1 loops
    /// between `loop_start` and `loop_end` when the body loops (`0x5b441b`).
    pub flags: u16,
    /// `+0x02`: work size in 2-byte units (read signed, `0x5b7446`).
    pub work_units: i16,
    /// `+0x04` (`0x5b758a`).
    pub lifetime: u16,
    /// `+0x06`: its own index (37328/37328).
    pub own_index: u16,
    /// `+0x09`: bit 0 set -> pass 0 (`+0x178`), clear -> pass 1 (`0x5b750c`). 2 everywhere.
    pub list_bit: u8,
    /// `+0x0a` (0 everywhere).
    pub b0a: u8,
    /// `+0x0b` matrix type -> instance `+0x90` (`0x5b766a`).
    pub matrix_type: u8,
    /// `+0x0c` / `+0x0e`.
    pub loop_start: u16,
    pub loop_end: u16,
    /// `+0x10`, `+0x11` (= lifetime & 0xff), `+0x12` (1 everywhere): no reader found.
    pub b10: u8,
    pub b11: u8,
    pub b12: u8,
    /// `+0x1c`: slot descriptors.
    pub slots: Vec<Slot>,
    /// `+0x20`: init list.
    pub init: CmdList,
    /// `+0x24`: key lists.
    pub keys: Vec<CmdList>,
    /// `+0x28..+0x44`: the 8 slot lists (counts at `+0x14..+0x1b`): update, draw, release, loop,
    /// (unused), first update, after-pass, slot 8 (README 6.4).
    pub lists: [Vec<u16>; 8],
}

impl Part {
    /// Work bytes (`(2 * v)`, `0x5b7446`); 0 if negative.
    pub fn work_bytes(&self) -> usize {
        (self.work_units as i32 * 2).max(0) as usize
    }
}

/// Slot descriptor: `u16 command id, u16 work / 2, u16 n, u16 0, s32 -> u16 slot[n], u16 work[n]`.
#[derive(Debug, Clone)]
pub struct Slot {
    pub id: u16,
    /// Work offset in 2-byte units: work = part work + `(2 * v) & 0xffff` (`0x5b5782`).
    pub work: u16,
    /// The earlier slots it reads, with their work offsets (2-byte units).
    pub inputs: Vec<Input>,
}

impl Slot {
    pub fn work_offset(&self) -> usize {
        (self.work as usize * 2) & 0xffff
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub slot: u16,
    pub work: u16,
}

/// Init list or key list: `s16 frame, s16 smallest span, s32 count, s32 -> entries`.
#[derive(Debug, Clone)]
pub struct CmdList {
    /// File offset of the list header (parameter offsets count from here).
    pub offset: usize,
    pub frame: i16,
    pub min_span: i16,
    pub entries: Vec<Entry>,
}

/// Entry: `u16 slot, u16 parameter offset from the list, s16 start frame, s16 span`.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub slot: u16,
    pub param: u16,
    pub start: i16,
    pub span: i16,
}

impl CmdList {
    /// File offset of an entry's parameter block.
    pub fn params(&self, e: &Entry) -> usize {
        self.offset + e.param as usize
    }
}

/// The four resource sections and the type code their ids carry (README 9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResKind {
    Tex = 0,
    Anm = 1,
    Mdl = 2,
    Vtl = 3,
}

/// Resource record (24 bytes): `{f32 1.0, u32 id, u32 size, u32 file offset, i32 -1, u32 0}`.
#[derive(Debug, Clone)]
pub struct Resource {
    pub kind: ResKind,
    pub index: usize,
    /// `+0x00` (1.0; 0.95 in 3 records game-wide; never read).
    pub scale: f32,
    pub id: u32,
    pub size: u32,
    pub offset: u32,
}

impl Resource {
    /// `(id & 0x78000000) == 0x70000000`: lives in resident.frr, found by id (`0x5b3076`).
    pub fn is_resident(&self) -> bool {
        self.id & 0x7800_0000 == 0x7000_0000
    }
}

/// A parsed FEP. The bytes stay with it: parameter blocks and resources are read from them.
#[derive(Debug, Clone)]
pub struct Fep {
    pub data: Vec<u8>,
    pub header: Header,
    pub names: Vec<Name>,
    pub groups: Vec<Group>,
    pub bodies: Vec<Body>,
    pub tex: Vec<Resource>,
    pub mdl: Vec<Resource>,
    pub anm: Vec<Resource>,
    pub vtl: Vec<Resource>,
    /// Section 6 record offsets ("special" elements; empty in every file; not decoded).
    pub special: Vec<usize>,
}

impl Fep {
    pub fn parse(data: Vec<u8>) -> Result<Fep> {
        let b = Bytes(&data);
        if b.slice(0, 3, "magic")? != b"FEP" {
            return Err(Error::NotFep);
        }
        let (v0, v1) = (b.i16(4, "version")?, b.i16(6, "version")?);
        if (v0, v1) != (1, 9) {
            return Err(Error::Version(v0, v1));
        }
        check_signatures(b)?;
        let mut counts = [0u16; 6];
        for (k, c) in counts.iter_mut().enumerate() {
            *c = b.u16(0x1c + 2 * k, "header counts")?;
        }
        let mut sections = [0u32; 8];
        for (k, s) in sections.iter_mut().enumerate() {
            *s = b.u32(0x28 + 4 * k, "header sections")?;
        }
        let header = Header {
            byte3: b.u8(3, "magic")?,
            id_base: b.u32(0x10, "header")?,
            file_size: b.u32(0x14, "header")?,
            category: b.u16(0x18, "header")? & 0xf,
            counts,
            sections,
        };

        // Section 7: names.
        let s7 = sections[7] as usize;
        let n_names = b.i32(s7, "names")?.max(0) as usize;
        let names_at = b.u32(s7 + 4, "names")? as usize;
        let mut names = Vec::with_capacity(n_names);
        for k in 0..n_names {
            let r = names_at + 12 * k;
            let (id, len, at) = (b.u32(r, "name")?, b.u32(r + 4, "name")? as usize, b.u32(r + 8, "name")? as usize);
            let name = String::from_utf8_lossy(b.slice(at, len, "name text")?).into_owned();
            names.push(Name { id, name });
        }

        // Section 0: groups and their elements.
        let mut groups = Vec::new();
        for at in b.offsets(sections[0] as usize, counts[0] as usize, "groups")? {
            let (id, body, n) = (b.u32(at, "group")?, b.u16(at + 4, "group")?, b.u16(at + 6, "group")? as usize);
            let table = b.u32(at + 8, "group")? as usize;
            let mut elements = Vec::with_capacity(n);
            for e in b.offsets(table, n, "group elements")? {
                elements.push(element(b, e)?);
            }
            groups.push(Group { id, body, elements });
        }

        // Section 1: bodies.
        let mut bodies = Vec::new();
        for at in b.offsets(sections[1] as usize, counts[1] as usize, "bodies")? {
            bodies.push(body(b, at)?);
        }

        // Sections 2..5: resources.
        let res = |section: usize, kind: ResKind| -> Result<Vec<Resource>> {
            let n = counts[section] as usize;
            b.offsets(sections[section] as usize, n, "resources")?
                .into_iter()
                .enumerate()
                .map(|(index, r)| {
                    Ok(Resource {
                        kind,
                        index,
                        scale: b.f32(r, "resource")?,
                        id: b.u32(r + 4, "resource")?,
                        size: b.u32(r + 8, "resource")?,
                        offset: b.u32(r + 12, "resource")?,
                    })
                })
                .collect()
        };
        let (tex, mdl, anm, vtl) = (res(2, ResKind::Tex)?, res(3, ResKind::Mdl)?, res(4, ResKind::Anm)?, res(5, ResKind::Vtl)?);

        // Section 6: {u16 count, u16 garbage, u32 -> u32[count]}.
        let s6 = sections[6] as usize;
        let n6 = b.u16(s6, "special")? as usize;
        let special = if n6 == 0 { Vec::new() } else { b.offsets(b.u32(s6 + 4, "special")? as usize, n6, "special")? };

        Ok(Fep { header, names, groups, bodies, tex, mdl, anm, vtl, special, data })
    }

    /// Section-7 record index of a name (`0x5abb70`: length, then text).
    pub fn find(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n.name == name)
    }

    /// Group, element and body index of an element id (`0x5acb40`): group by `(id >> 10) & 0x7f`
    /// (`0x5acc96`), element by `id & 0x1ff` (`0x5accfc`), body = the group's `+4`.
    pub fn element_by_id(&self, id: u32) -> Result<(usize, usize, usize)> {
        if id & 0x200 != 0 {
            return Err(Error::Special(id));
        }
        let g = self
            .groups
            .iter()
            .position(|g| (g.id >> 10) & 0x7f == (id >> 10) & 0x7f)
            .ok_or_else(|| Error::NoElement(format!("{id:#x}")))?;
        let e = self.groups[g]
            .elements
            .iter()
            .position(|e| e.id & 0x1ff == id & 0x1ff)
            .ok_or_else(|| Error::NoElement(format!("{id:#x}")))?;
        let body = self.groups[g].body as usize;
        if body >= self.bodies.len() {
            return Err(Error::NoElement(format!("{id:#x} (body {body})")));
        }
        Ok((g, e, body))
    }

    /// Group, element and body index of an element by name.
    pub fn element(&self, name: &str) -> Result<(usize, usize, usize)> {
        let k = self.find(name).ok_or_else(|| Error::NoElement(name.into()))?;
        self.element_by_id(self.names[k].id)
    }

    /// Resource record of a section by index.
    pub fn resource(&self, kind: ResKind, index: usize) -> Option<&Resource> {
        match kind {
            ResKind::Tex => &self.tex,
            ResKind::Anm => &self.anm,
            ResKind::Mdl => &self.mdl,
            ResKind::Vtl => &self.vtl,
        }
        .get(index)
    }

    /// Resource record by id, in any section.
    pub fn resource_by_id(&self, id: u32) -> Option<&Resource> {
        self.tex.iter().chain(&self.anm).chain(&self.mdl).chain(&self.vtl).find(|r| r.id == id)
    }

    /// A local resource's bytes (`file + offset`, `size` long, clipped to the file); `None` for an
    /// id-0 record (skipped by DDD, `0x5b3076`) and for resident ones (see `Resident::get`).
    pub fn local_data(&self, r: &Resource) -> Option<&[u8]> {
        if r.id == 0 || r.is_resident() {
            return None;
        }
        let at = r.offset as usize;
        self.data.get(at..(at + r.size as usize).min(self.data.len()))
    }

    /// The id DDD gives a local resource (README 9.1; 5044/5044 records game-wide).
    pub fn local_id(&self, kind: ResKind, index: usize) -> u32 {
        let h = self.header.id_base;
        0x8000_0000 | (h >> 28) << 27 | ((h >> 24) & 0xf) << 16 | ((h >> 16) & 0xff) << 8 | (kind as u32) << 7 | index as u32
    }

    /// A parameter block: from its file offset, `sig::param_size(id)` bytes (clipped to the file).
    pub fn params(&self, at: usize, id: u16) -> &[u8] {
        let end = (at + sig::param_size(id)).min(self.data.len());
        self.data.get(at..end).unwrap_or(&[])
    }
}

/// Header `+0x08` -> `{u32 158, u32 offset}` -> 158 x 8 bytes, which must equal the exe's.
/// (DDD checks only the ids a body uses, as u32 words indexed `id * 4`, at element creation:
/// `0x5ac2c2..0x5ac30a`. We refuse the whole file instead; every file of the game passes.)
fn check_signatures(b: Bytes) -> Result<()> {
    let at = b.u32(8, "signature table")? as usize;
    if b.u32(at, "signature table")? as usize != sig::COUNT {
        return Err(Error::Signature(0));
    }
    let table = b.u32(at + 4, "signature table")? as usize;
    let bytes = b.slice(table, sig::COUNT * 8, "signature table")?;
    match sig::TABLE.iter().zip(bytes.chunks(8)).position(|(a, b)| a[..] != *b) {
        Some(id) => Err(Error::Signature(id)),
        None => Ok(()),
    }
}

fn element(b: Bytes, at: usize) -> Result<Element> {
    b.slice(at, 0x9c, "element")?;
    Ok(Element {
        offset: at,
        id: b.u32(at, "element")?,
        view_angle: b.u8(at + 6, "element")?,
        b07: b.u8(at + 7, "element")?,
        view_bounds: b.u8(at + 0x0a, "element")?,
        distance: b.u8(at + 0x0b, "element")?,
        start_offset: b.vec4(at + 0x10, "element")?,
        v30: b.vec4(at + 0x30, "element")?,
        b8e: b.u8(at + 0x8e, "element")?,
        start_delay: b.u16(at + 0x90, "element")?,
    })
}

fn body(b: Bytes, at: usize) -> Result<Body> {
    b.slice(at, 0x34, "body")?;
    let n_events = b.u16(at + 0x10, "body")? as usize;
    let n_parts = b.u16(at + 0x12, "body")? as usize;
    let n_ids = b.u16(at + 0x14, "body")? as usize;
    let n_28 = b.u16(at + 0x16, "body")? as usize;
    let n_groups = b.u16(at + 0x18, "body")? as usize;
    let table = |field: usize, n: usize, what| -> Result<Vec<usize>> {
        let t = b.u32(at + field, what)? as usize;
        if t == 0 {
            Ok(vec![0; n])
        } else {
            b.offsets(t, n, what)
        }
    };

    let mut events = Vec::with_capacity(n_events);
    for e in table(0x1c, n_events, "spawn events")? {
        events.push(if e == 0 {
            None
        } else {
            let n = b.i16(e + 2, "spawn event")?.max(0) as usize;
            Some(SpawnEvent { frame: b.i16(e, "spawn event")?, refs: b.u16s(b.u32(e + 4, "spawn event")? as usize, n, "spawn refs")? })
        });
    }
    let mut parts = Vec::with_capacity(n_parts);
    for p in table(0x20, n_parts, "parts")? {
        parts.push(if p == 0 { None } else { Some(part(b, p)?) });
    }
    let ids_at = b.u32(at + 0x24, "body")? as usize;
    let command_ids = if ids_at == 0 { Vec::new() } else { b.u16s(ids_at, n_ids, "command ids")? };
    // `+0x28` -> u32[n] offsets of `{u32 count, u32 -> u16[count]}` records.
    let mut lists28 = Vec::new();
    for r in table(0x28, n_28, "+0x28 lists")? {
        if r == 0 {
            lists28.push(Vec::new());
            continue;
        }
        let n = b.u32(r, "+0x28 list")? as usize;
        let o = b.u32(r + 4, "+0x28 list")? as usize;
        lists28.push(if o == 0 || n == 0 { Vec::new() } else { b.u16s(o, n.min(0x1000), "+0x28 list")? });
    }
    let mut part_groups = Vec::with_capacity(n_groups);
    for g in table(0x2c, n_groups, "part groups")? {
        if g == 0 {
            part_groups.push(Vec::new());
            continue;
        }
        let n = b.u16(g, "part group")? as usize;
        let r = b.u32(g + 4, "part group")? as usize;
        let mut recs = Vec::with_capacity(n);
        for k in 0..n {
            if r == 0 {
                break;
            }
            recs.push((b.i16(r + 4 * k, "part group")?, b.u16(r + 4 * k + 2, "part group")?));
        }
        part_groups.push(recs);
    }
    Ok(Body {
        offset: at,
        flags: b.u16(at, "body")?,
        u02: b.u16(at + 2, "body")?,
        length: b.i32(at + 4, "body")?,
        u08: b.u16(at + 8, "body")?,
        kind: b.u8(at + 0x0a, "body")?,
        b0b: b.u8(at + 0x0b, "body")?,
        stop_mode: b.u32(at + 0x0c, "body")?,
        n30: b.u16(at + 0x1a, "body")?,
        events,
        parts,
        command_ids,
        lists28,
        part_groups,
    })
}

fn part(b: Bytes, at: usize) -> Result<Part> {
    b.slice(at, 0x48, "part")?;
    let n_slots = b.u8(at + 8, "part")? as usize;
    let n_keys = b.u8(at + 0x13, "part")? as usize;
    let bad = |slot: u16| Error::BadSlot { part_offset: at, slot };

    let mut slots = Vec::with_capacity(n_slots);
    let st = b.u32(at + 0x1c, "part")? as usize;
    for k in 0..n_slots {
        let s = if st == 0 { 0 } else { b.u32(st + 4 * k, "slot table")? as usize };
        if s == 0 {
            return Err(Error::Truncated("slot descriptor (null)", at));
        }
        let id = b.u16(s, "slot")?;
        if id as usize >= sig::COUNT {
            return Err(Error::BadCommand(id));
        }
        let n = b.u16(s + 4, "slot")? as usize;
        let io = b.u32(s + 8, "slot")? as usize;
        let (sl, wk) = (b.u16s(io, n, "slot inputs")?, b.u16s(if io == 0 { 0 } else { io + 2 * n }, n, "slot inputs")?);
        let inputs: Vec<Input> = sl.into_iter().zip(wk).map(|(slot, work)| Input { slot, work }).collect();
        if let Some(i) = inputs.iter().find(|i| i.slot as usize >= n_slots) {
            return Err(bad(i.slot));
        }
        slots.push(Slot { id, work: b.u16(s + 2, "slot")?, inputs });
    }

    let list = |o: usize| -> Result<CmdList> {
        if o == 0 {
            return Err(Error::Truncated("command list (null)", at));
        }
        let n = b.i32(o + 4, "command list")?.max(0) as usize;
        let eo = b.u32(o + 8, "command list")? as usize;
        let mut entries = Vec::with_capacity(n);
        for k in 0..n {
            let e = eo + 8 * k;
            let entry = Entry { slot: b.u16(e, "entry")?, param: b.u16(e + 2, "entry")?, start: b.i16(e + 4, "entry")?, span: b.i16(e + 6, "entry")? };
            if entry.slot as usize >= n_slots {
                return Err(bad(entry.slot));
            }
            entries.push(entry);
        }
        Ok(CmdList { offset: o, frame: b.i16(o, "command list")?, min_span: b.i16(o + 2, "command list")?, entries })
    };
    let init = list(b.u32(at + 0x20, "part")? as usize)?;
    let kt = b.u32(at + 0x24, "part")? as usize;
    let mut keys = Vec::with_capacity(n_keys);
    for k in 0..n_keys {
        keys.push(list(if kt == 0 { 0 } else { b.u32(kt + 4 * k, "key lists")? as usize })?);
    }
    let mut lists: [Vec<u16>; 8] = Default::default();
    for (k, l) in lists.iter_mut().enumerate() {
        let n = b.u8(at + 0x14 + k, "part")? as usize;
        let o = b.u32(at + 0x28 + 4 * k, "part")? as usize;
        *l = if n == 0 { Vec::new() } else { b.u16s(o, n, "slot list")? };
        if let Some(&s) = l.iter().find(|&&s| s as usize >= n_slots) {
            return Err(bad(s));
        }
    }
    Ok(Part {
        offset: at,
        flags: b.u16(at, "part")?,
        work_units: b.i16(at + 2, "part")?,
        lifetime: b.u16(at + 4, "part")?,
        own_index: b.u16(at + 6, "part")?,
        list_bit: b.u8(at + 9, "part")?,
        b0a: b.u8(at + 0x0a, "part")?,
        matrix_type: b.u8(at + 0x0b, "part")?,
        loop_start: b.u16(at + 0x0c, "part")?,
        loop_end: b.u16(at + 0x0e, "part")?,
        b10: b.u8(at + 0x10, "part")?,
        b11: b.u8(at + 0x11, "part")?,
        b12: b.u8(at + 0x12, "part")?,
        slots,
        init,
        keys,
        lists,
    })
}
