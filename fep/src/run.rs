//! The interpreter: one element instance, its clock, its parts and the order DDD runs them in
//! (docs/fep/README.md section 7), read from the exe:
//!
//! - creation `0x5ac210` (start frame, start delay, catch-up time step `0x5ac74c..0x5ac826`);
//! - the time step `0x5b4290` (instance clock, part clocks, loop wrap flag, death, spawn events,
//!   body end, stop request and fade);
//! - the pass-1 update `0x5b4700` (first-update slots, keys `0x5b5040` -> `0x5b5180`, update
//!   slots, through the walker `0x5b5f10`) and the after-pass `0x5b5e70`;
//! - loop rewind `0x5b6680` with its key search `0x5b80c0`;
//! - spawning `0x5b73d0` (init `0x5b56f0`, frame owner) and death `0x5b82b0`;
//! - the end `0x5ad370` and the stop request `0x5b9cd0`.
//!
//! A part instance keeps DDD's fields that matter (README 7.3); its work area is a byte array
//! the commands read and write as DDD's do. Instances of parts live in a slab: an index stays
//! valid while the part lives, and a parent is never freed before its children (`0x5b82b0`
//! keeps a part with live children), so `parent` / `owner` indices are always live.
//!
//! What a frame is: `step(dt)` = the time task's step (`0x5b55d0` -> `0x5b4290`), the pass-0
//! update (no file uses it), the pass-1 update and the after-pass, in that order (README 7.1;
//! the task order is INFERRED from their priorities).

use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::cmd::{self, ClassStatus};
use crate::parse::{Entry, Error, Fep, Part, Result};
use crate::res::Resident;
use crate::rng::{Rng, XorShift};
use crate::sig;

// Part instance state bits (`+0x02`, README 7.3).
/// Has looped (`0x5b50ee`): Rnd* and Mdlts_plus skip their randoms from then on.
pub(crate) const LOOPED: u16 = 2;
/// First-update slots done (`0x5b606e`).
const STARTED: u16 = 4;
/// Dead, kept for its children (`0x5b82db`).
const DEAD: u16 = 8;
/// Loop wrap pending (`0x5b444c`).
const WRAP: u16 = 0x10;
/// Spawned by a ref with bit 13: waits for its held ancestors (`0x5b7549`, `0x5b82f9`).
const WAITS: u16 = 0x40;
/// Holds such a child (`0x5b7541`).
const HOLDS: u16 = 0x80;

// Instance flags `+0x02`.
/// Told to stop by a body with stop mode 3: loops stop looping (`0x5b9d6d`).
const STOPPING: u16 = 0x10;
/// Remove at the next time step (`0x5b9d51`, `0x5b42ae`).
const KILL: u16 = 0x20;

/// Spawn info (`ctx+0x98`, read by slot 0): flags, then a vec4 per bit (README 7.3): bits 0-2
/// position / velocity / acceleration, 3-5 rotation, 6-8 scale, 9-11 colour.
#[derive(Debug, Clone, Copy)]
pub struct SpawnInfo {
    pub flags: u32,
    pub v: [Vec4; 12],
}

impl SpawnInfo {
    /// The vec4 of a bit, if its flag is set.
    pub fn get(&self, bit: usize) -> Option<Vec4> {
        (bit < 12 && self.flags & (1 << bit) != 0).then(|| self.v[bit])
    }
}

/// A part instance (`0x5b73d0`; README 7.3).
#[derive(Debug, Clone)]
pub(crate) struct PartInst {
    pub alive: bool,
    /// Part index in the body (`+0x32`).
    pub index: usize,
    /// `+0x02`.
    pub state: u16,
    /// Part flags `+0x04` (part `+0x00`).
    pub flags: u16,
    /// Pass 0 or 1 (`+0x08`).
    pub pass: u8,
    /// `+0x20`: the generator part that spawned it.
    pub parent: Option<usize>,
    /// `+0x28`: whose own frame (`+0x88`) is its frame; `None` = the instance's matrix.
    pub owner: Option<usize>,
    /// `+0x30`: live children.
    pub children: u16,
    /// `+0x40`: time in frames.
    pub time: f32,
    /// `+0x44`.
    pub lifetime: u16,
    /// `+0x46`: next key list, -1 none.
    pub next_key: i16,
    /// The work area (`+0x60` -> `self + 0xa0`), zeroed (INFERRED: `0x5bd400` zeroes).
    pub work: Vec<u8>,
    /// `+0x68`: file offset of each slot's current parameter block.
    pub params: Vec<Option<usize>>,
    /// `+0x70`: each slot's current key entry.
    pub current: Vec<Option<Entry>>,
    /// `+0x78` local, `+0x80` world, `+0x88` own frame (types 2, 6..9 only); none for type 0xff.
    pub local: Option<Mat4>,
    pub world: Option<Mat4>,
    pub frame: Option<Mat4>,
    /// `+0x90` (part `+0x0b`).
    pub matrix_type: u8,
    /// `+0x91`: FecLmatrix param byte 1, or the parent's.
    pub b91: u8,
    /// Per slot: a FecMegaanm_plus sprite buffer (DDD keeps its pointer at work `+0`).
    pub mega: Vec<Option<Box<cmd::Mega>>>,
    /// Per slot: a FecLocus_tail_strategy object, a FecShapetail state (DDD: pointers at work `+0`).
    pub locus: Vec<Option<Box<cmd::Locus>>>,
    pub tail: Vec<Option<Box<cmd::Tail>>>,
    /// Per slot: a FecWave_bindlocus_strategy object, a FecBindlocus_polygon state, a line
    /// particle pool (FecEllipse_line_particle / FecRadialg_line_particle) (DDD: pointers at work
    /// `+0`).
    pub wave: Vec<Option<Box<cmd::Wave>>>,
    pub ribbon: Vec<Option<Box<cmd::Ribbon>>>,
    pub lines: Vec<Option<Box<cmd::Lines>>>,
    /// Per slot: FecFlash's value and rectangle (DDD: a pointer at work `+0`).
    pub flash: Vec<Option<cmd::Flash>>,
}

macro_rules! rw {
    ($rd:ident, $wr:ident, $t:ty, $n:expr) => {
        pub fn $rd(&self, at: usize) -> $t {
            self.work.get(at..at + $n).map_or(0 as $t, |b| <$t>::from_le_bytes(b.try_into().unwrap_or([0; $n])))
        }
        pub fn $wr(&mut self, at: usize, v: $t) {
            if let Some(b) = self.work.get_mut(at..at + $n) {
                b.copy_from_slice(&v.to_le_bytes());
            }
        }
    };
}

impl PartInst {
    rw!(rd, wr, f32, 4);
    rw!(rd_u16, wr_u16, u16, 2);
    rw!(rd_u8, wr_u8, u8, 1);

    pub fn rd4(&self, at: usize) -> Vec4 {
        Vec4::new(self.rd(at), self.rd(at + 4), self.rd(at + 8), self.rd(at + 12))
    }
    pub fn wr4(&mut self, at: usize, v: Vec4) {
        self.wr3(at, v.xyz());
        self.wr(at + 12, v.w);
    }
    /// xyz only; w kept.
    pub fn wr3(&mut self, at: usize, v: Vec3) {
        self.wr(at, v.x);
        self.wr(at + 4, v.y);
        self.wr(at + 8, v.z);
    }
    pub fn rd2(&self, at: usize) -> glam::Vec2 {
        glam::Vec2::new(self.rd(at), self.rd(at + 4))
    }
    pub fn wr2(&mut self, at: usize, v: glam::Vec2) {
        self.wr(at, v.x);
        self.wr(at + 4, v.y);
    }
}

/// How an instance is started: the random source, resident.frr (for resident ANMs, read by
/// FecAnm_plus's clock), the instance matrix (`+0x60`, see `Instance::set_matrix`), and
/// `world_unit`: how many units of the space the instance matrix maps into make one DDD world
/// unit (1 in DDD). Not DDD's: a world-space trail (FecLocus_tail_strategy `+0x26`) measures its
/// spacing in that space, so a caller whose matrix also converts units (DDD -> KH1 x100) says so
/// here.
pub struct Options {
    pub rng: Box<dyn Rng + Send>,
    pub resident: Option<Arc<Resident>>,
    pub matrix: Mat4,
    pub world_unit: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options { rng: Box::new(XorShift::default()), resident: None, matrix: Mat4::IDENTITY, world_unit: 1.0 }
    }
}

/// An element instance (`0x5ac210`, 0x1d0 bytes in DDD; README 7.3).
pub struct Instance {
    pub(crate) fep: Arc<Fep>,
    pub(crate) resident: Option<Arc<Resident>>,
    pub(crate) rng: Box<dyn Rng + Send>,
    /// Body / group / element indices.
    pub(crate) body: usize,
    group: usize,
    element: usize,
    /// The slab of part instances and the two pass lists (`+0x178`, `+0x180`), in link order.
    pub(crate) parts: Vec<PartInst>,
    free: Vec<usize>,
    lists: [Vec<usize>; 2],
    /// `+0x60`: the instance matrix (DDD rebuilds it each pass-1 frame from the attachment,
    /// `0x5ae260`; here the caller sets it). DDD's row-vector matrix, stored as glam stores
    /// it: DDD's `v * M` is glam's `M * v`.
    pub(crate) effect: Mat4,
    /// `Options::world_unit`.
    pub(crate) world_unit: f32,
    /// `+0x04` body flags, `+0x02` instance flags.
    body_flags: u16,
    flags: u16,
    /// `+0x138` clock, `+0x13c` this step's dt, `+0x144` stop time (-1 none), `+0x148` fade.
    time: f32,
    pub(crate) dt: f32,
    kill_timer: f32,
    fade: f32,
    /// `+0x16c` length, `+0x1b0` next spawn event (-1 none).
    length: i32,
    next_event: i16,
    ended: bool,
    // Context fields (`0xaa3de0`): `+0x90` phase, `+0x98` spawn info, `+0xa0` key lists run.
    phase: u8,
    pub(crate) spawn_info: Option<SpawnInfo>,
    pub(crate) keys_run: u32,
}

impl Instance {
    /// Start an element by name, with default options (`Options::default`).
    pub fn start(fep: Arc<Fep>, name: &str, start_frame: f32) -> Result<Instance> {
        Self::start_with(fep, name, start_frame, Options::default())
    }

    /// Start an element by name (`0x5abb70` name, `0x5acb40` id -> group / element / body,
    /// `0x5ac210` creation). Fails when a command id is past the table (`0x5ac2e8`; the table
    /// itself was compared at parse) or the start frame is at or past the body's length
    /// (`0x5ac30c`).
    ///
    /// The instance is created as if between DDD's time task and its pass-1 update (creation
    /// phase 1 or 5, `0x5ac762..0x5ac80f`; INFERRED: where motion events start effects): the
    /// clock starts at minus the element's start delay (`0x5ac4eb`), less 1, and one catch-up
    /// time step of `start_frame + 1` runs at once; then this frame's pass-1 update runs. So at
    /// start frame 0 the clock reads 0 and the spawn events of frame 0 have fired.
    pub fn start_with(fep: Arc<Fep>, name: &str, start_frame: f32, opts: Options) -> Result<Instance> {
        let (group, element, body) = fep.element(name)?;
        let b = &fep.bodies[body];
        if let Some(&id) = b.command_ids.iter().find(|&&id| id as usize >= sig::COUNT) {
            return Err(Error::BadCommand(id));
        }
        // `comiss start, length; jb ok`: start < length, else no instance.
        if !(start_frame < b.length as f32) {
            return Err(Error::StartPastEnd { start: start_frame, length: b.length });
        }
        let delay = fep.groups[group].elements[element].start_delay;
        let mut i = Instance {
            body_flags: b.flags,
            length: b.length,
            fep: fep.clone(),
            resident: opts.resident,
            rng: opts.rng,
            body,
            group,
            element,
            parts: Vec::new(),
            free: Vec::new(),
            lists: [Vec::new(), Vec::new()],
            effect: opts.matrix,
            world_unit: opts.world_unit,
            flags: 0,
            time: -(delay as f32),
            dt: 0.0,
            kill_timer: -1.0,
            fade: 1.0,
            next_event: 0,
            ended: false,
            phase: 1,
            spawn_info: None,
            keys_run: 0,
        };
        // Catch-up (`0x5ac7ed`): clock - 1, time step of start + 1.
        i.time -= 1.0;
        i.time_step(start_frame + 1.0);
        i.update();
        Ok(i)
    }

    /// One frame of `dt` FEP frames (DDD: half the game's frame delta, `0x554fbd`; GUESS 1.0 =
    /// 1/30 s): the time step, then the updates. dt 0 stops the clock and skips the keys, as in
    /// DDD (`0x5b42f4`, `0x5b5056`).
    pub fn step(&mut self, dt: f32) {
        if self.ended {
            return;
        }
        self.phase = 0;
        self.time_step(dt);
        self.phase = 1;
        if !self.ended {
            self.update();
        }
    }

    /// The instance matrix (`+0x60`) the parts' frames come from, in DDD's convention (row
    /// vectors; stored as glam stores a matrix, so glam's `M * v` is DDD's `v * M`). DDD makes
    /// it each pass-1 frame from the instance's position / rotation / scale and the bone it is
    /// attached to (`0x5ae260`, not decoded); set it before each `step`. Parts of matrix type
    /// 0, 1, 3, 4, 5 follow it live; types 2 and 6..8 copy it once (README 7.3).
    pub fn set_matrix(&mut self, m: Mat4) {
        self.effect = m;
    }

    /// A stop request (`0x5b9cd0`): the body's stop mode decides (`0x5b43a7`): 0 / 1 fade
    /// over 30 / 15 frames then end, 3 let it play on with its loops released, other end at
    /// the next step. A second request does nothing.
    pub fn stop(&mut self) {
        if self.ended || self.kill_timer >= 0.0 {
            return;
        }
        if 0.0 > self.time {
            self.flags |= KILL;
        } else {
            self.kill_timer = self.time;
        }
        if self.fep.bodies[self.body].stop_mode == 3 {
            self.flags |= STOPPING;
        }
    }

    /// Ended (`0x5ad370` ran): no parts left; `step` does nothing.
    pub fn is_ended(&self) -> bool {
        self.ended
    }

    /// The clock (`+0x138`), in frames.
    pub fn time(&self) -> f32 {
        self.time
    }

    /// The body's length (`+0x16c`).
    pub fn length(&self) -> i32 {
        self.length
    }

    /// The fade factor (`+0x148`): 1, falling to 0 over a stop fade (stop modes 0 / 1). How DDD
    /// applies it to the drawing is not decoded (README 13.6).
    pub fn fade(&self) -> f32 {
        self.fade
    }

    /// The body's index, and the element's group / element index.
    pub fn body_index(&self) -> usize {
        self.body
    }
    pub fn element_index(&self) -> (usize, usize) {
        (self.group, self.element)
    }

    pub fn fep(&self) -> &Arc<Fep> {
        &self.fep
    }

    /// The live parts (pass 0's list, then pass 1's, each in link order), without the dead
    /// ones that wait for their children (DDD does not draw them, `0x5b59ea`).
    pub fn parts(&self) -> impl Iterator<Item = PartView<'_>> + '_ {
        self.lists.iter().flatten().copied().filter(|&p| self.parts[p].state & DEAD == 0).map(move |p| PartView { i: self, pid: p })
    }

    /// Spawn a part of the body as a root, as a spawn event does (`0x5b73d0` with no parent and
    /// no spawn info): `part` = its index (a part ref: bit 13 = hold, README 5), `start_time`
    /// = its clock. Its keys run at the next step's update. For tests and for parts DDD spawns
    /// through a class not implemented here (FecGenerateanchorvertex).
    pub fn spawn_root(&mut self, part: u16, start_time: f32) -> Option<usize> {
        if self.ended {
            return None;
        }
        let saved = self.phase;
        self.phase = 1;
        let r = self.spawn(None, part as i32, start_time, None);
        self.phase = saved;
        r
    }

    /// Every allocated part instance, dead ones included.
    pub fn part_count(&self) -> usize {
        self.lists[0].len() + self.lists[1].len()
    }

    /// The part definition of a part instance.
    pub(crate) fn def(&self, pid: usize) -> Option<&Part> {
        self.fep.bodies.get(self.body)?.parts.get(self.parts.get(pid)?.index)?.as_ref()
    }

    // --- the time step (`0x5b4290`) ---

    fn time_step(&mut self, dt: f32) {
        if self.ended {
            return;
        }
        if self.flags & KILL != 0 {
            return self.end();
        }
        if dt == 0.0 {
            self.dt = 0.0;
            return;
        }
        let looping = self.body_flags & 2 != 0;
        let mut dt = dt;
        // A looping effect not stopping takes a long frame as 1 (`0x5b4361`, `[0x867ee4]` 10).
        if looping && 0.0 > self.kill_timer && dt > 10.0 {
            dt = 1.0;
        }
        self.dt = dt;
        self.time += dt;
        if self.kill_timer >= 0.0 {
            let n = match self.fep.bodies[self.body].stop_mode {
                0 => 30.0, // `[0x867efc]`
                1 => 15.0, // `[0x867ef0]`
                3 => 0.0,
                _ => return self.end(),
            };
            if n > 0.0 {
                let elapsed = self.time - self.kill_timer;
                self.fade = 1.0 - elapsed / n;
                if elapsed >= n || 0.0 > elapsed {
                    return self.end();
                }
            }
        }

        // Every part's clock (`0x5b4400`), the loop flag and death.
        let stopping = self.flags & STOPPING != 0;
        for g in 0..2 {
            let mut k = 0;
            while k < self.lists[g].len() {
                let pid = self.lists[g][k];
                let p = &mut self.parts[pid];
                p.time += dt;
                if 0.0 > p.time {
                    k += 1;
                    continue;
                }
                let f = p.time as i32;
                if p.flags & 2 != 0 && looping && !stopping {
                    let end = self.def(pid).map_or(0, |d| d.loop_end) as f32;
                    let p = &mut self.parts[pid];
                    if p.time >= end {
                        p.state |= WRAP;
                    }
                    k += 1;
                    continue;
                }
                if p.lifetime as i32 > f || (p.flags & 1 != 0 && p.children != 0) {
                    k += 1;
                    continue;
                }
                if !self.kill_part(pid, false) {
                    k += 1;
                }
            }
        }

        let time = self.time;
        if 0.0 > time {
            return;
        }
        let length = self.length as f32;
        if length > time || time - self.dt == 0.0 {
            self.spawn_events(time);
        } else if if looping {
            self.flags & STOPPING != 0 && self.part_count() == 0
        } else {
            self.body_flags & 1 == 0 || self.part_count() == 0
        } {
            self.end();
        }
    }

    /// Spawn events (`0x5b45b2..0x5b46a3`): from the next one, while its frame <= int(clock),
    /// spawn its parts as roots with start time clock - frame; stop at the first future one.
    fn spawn_events(&mut self, time: f32) {
        let fep = self.fep.clone();
        let events = &fep.bodies[self.body].events;
        let mut ev = self.next_event as i32;
        let f = time as i32;
        while ev >= 0 {
            let Some(Some(e)) = events.get(ev as usize) else {
                ev = -1;
                break;
            };
            if e.frame as i32 > f {
                break;
            }
            let start = time - e.frame as f32;
            for &r in &e.refs {
                self.spawn(None, r as i32, start, None);
            }
            ev += 1;
            if ev as usize >= events.len() {
                ev = -1;
            }
        }
        self.next_event = ev as i16;
    }

    /// The end (`0x5ad370`): every part freed (release slots run), the instance gone.
    fn end(&mut self) {
        for g in 0..2 {
            while let Some(&pid) = self.lists[g].first() {
                self.kill_part(pid, true);
            }
        }
        self.ended = true;
    }

    // --- spawning and death ---

    /// Spawn a part (`0x5b73d0`): `r` = part ref (bits 0-11 index, bit 13 the hold bit),
    /// `time` = its start time, `info` = the spawn info for slot 0. Returns the new instance,
    /// or `None` for a negative ref, an empty part entry, a part with no slots (`0x5b7428`,
    /// `0x5b7440`) or a failed init.
    pub(crate) fn spawn(&mut self, parent: Option<usize>, r: i32, time: f32, info: Option<SpawnInfo>) -> Option<usize> {
        if r < 0 {
            return None;
        }
        let fep = self.fep.clone();
        let index = (r & 0xfff) as usize;
        let def = fep.bodies[self.body].parts.get(index)?.as_ref()?;
        if def.slots.is_empty() {
            return None;
        }
        let n = def.slots.len();
        // Matrices (`0x5b749f..0x5b74e6`): types 2 and 6..9 three, 0xff none, others two.
        let (local, world, frame) = match def.matrix_type {
            2 | 6..=9 => (Some(Mat4::IDENTITY), Some(Mat4::IDENTITY), Some(Mat4::IDENTITY)),
            0xff => (None, None, None),
            _ => (Some(Mat4::IDENTITY), Some(Mat4::IDENTITY), None),
        };
        let mut state = 0;
        if r & 0x2000 != 0 {
            if let Some(q) = parent {
                self.parts[q].state |= HOLDS;
            }
            state |= WAITS;
        }
        // Loopable part of a looping body starting past its loop end (`0x5b75e0..0x5b7605`).
        if def.flags & 2 != 0 && self.body_flags & 2 != 0 && time >= def.loop_end as f32 {
            state |= WRAP;
        }
        let part = PartInst {
            alive: true,
            index,
            state,
            flags: def.flags,
            pass: !def.list_bit & 1,
            parent,
            owner: None,
            children: 0,
            time,
            lifetime: def.lifetime,
            next_key: if def.keys.is_empty() { -1 } else { 0 },
            work: vec![0; def.work_bytes()],
            params: vec![None; n],
            current: vec![None; n],
            local,
            world,
            frame,
            matrix_type: def.matrix_type,
            b91: 0,
            mega: vec![None; n],
            locus: vec![None; n],
            tail: vec![None; n],
            wave: vec![None; n],
            ribbon: vec![None; n],
            lines: vec![None; n],
            flash: vec![None; n],
        };
        let pid = match self.free.pop() {
            Some(p) => {
                self.parts[p] = part;
                p
            }
            None => {
                self.parts.push(part);
                self.parts.len() - 1
            }
        };

        // Init (`0x5b56f0`): per init entry, its block becomes current, then slot 0. No class
        // here fails; DDD's clean-up on a failure (`0x5b57e8`) is not reproduced.
        let saved = self.spawn_info;
        self.spawn_info = info;
        let mut ok = true;
        for e in &def.init.entries {
            let s = e.slot as usize;
            let p = def.init.params(e);
            self.parts[pid].params[s] = Some(p);
            if !cmd::init(self, pid, s, p) {
                ok = false;
                break;
            }
        }
        self.spawn_info = saved;
        if !ok {
            self.parts[pid].alive = false;
            self.free.push(pid);
            return None;
        }

        let pass = self.parts[pid].pass as usize;
        self.lists[pass].push(pid);
        if let Some(q) = parent {
            self.parts[q].children = self.parts[q].children.wrapping_add(1);
        }
        self.capture_frame(pid, parent, def.matrix_type);
        // Spawned during the pass-1 update slots (phases 8, 9): its keys up to now run at once
        // (`0x5b793c..0x5b7961`).
        if (8..=9).contains(&self.phase) {
            let f = self.parts[pid].time as i32;
            self.keys(pid, f);
        }
        Some(pid)
    }

    /// Frame owner and own frame (`0x5b77bd..0x5b7935` at spawn with the matrix type,
    /// `0x5b85a0` from FecLmatrix's slot 6 with its byte 0; the same rule):
    /// - modes 2, 6, 7, 8: own frame = the parent's local x its frame (its owner's own frame,
    ///   else the instance matrix), or the instance matrix for a root; owner = itself; mode 2
    ///   then resets the rotation (`0x5b7916`), mode 7 normalises the axes (`0x598ae0`);
    /// - mode 9: own frame identity, owner itself;
    /// - other modes: the parent's owner (none for a root: the live instance matrix).
    ///
    /// A part without an own frame (its type is not 2 / 6..9) is left alone where DDD would
    /// write through a null pointer.
    pub(crate) fn capture_frame(&mut self, pid: usize, parent: Option<usize>, mode: u8) {
        let parent_owner = parent.and_then(|q| self.parts[q].owner);
        match mode {
            2 | 6..=8 => {
                if self.parts[pid].frame.is_none() {
                    return;
                }
                let mut f = match parent {
                    Some(q) => {
                        let src = match parent_owner {
                            Some(o) => self.parts[o].frame.unwrap_or(Mat4::IDENTITY),
                            None => self.effect,
                        };
                        // `0x58c5b0(dst, src, local)` = DDD `local * src` = glam `src * local`.
                        match self.parts[q].local {
                            Some(l) => src * l,
                            None => src,
                        }
                    }
                    None => self.effect,
                };
                if mode == 2 {
                    f.x_axis = Vec3::X.extend(f.x_axis.w);
                    f.y_axis = Vec3::Y.extend(f.y_axis.w);
                    f.z_axis = Vec3::Z.extend(f.z_axis.w);
                } else if mode == 7 {
                    let n = |v: Vec4| if v.length_squared() == 0.0 { v } else { v / v.length() };
                    f.x_axis = n(f.x_axis);
                    f.y_axis = n(f.y_axis);
                    f.z_axis = n(f.z_axis);
                }
                let p = &mut self.parts[pid];
                p.frame = Some(f);
                p.owner = Some(pid);
            }
            9 => {
                let p = &mut self.parts[pid];
                if p.frame.is_some() {
                    p.frame = Some(Mat4::IDENTITY);
                    p.owner = Some(pid);
                }
            }
            _ => self.parts[pid].owner = parent_owner,
        }
    }

    /// Death (`0x5b82b0`). Unforced (the time step): marked dead; kept while it has live
    /// children, or while a held ancestor (state 0x80) lives when it waits (0x40). Then (and
    /// always when forced, at the end): its release slots run for slots with a current block,
    /// the parent loses a child, it is unlinked and freed. Returns true when freed.
    fn kill_part(&mut self, pid: usize, force: bool) -> bool {
        if !force {
            self.parts[pid].state |= DEAD;
            if self.parts[pid].children > 0 {
                return false;
            }
            if self.parts[pid].state & WAITS != 0 {
                let mut q = self.parts[pid].parent;
                while let Some(a) = q {
                    let s = self.parts[a].state;
                    if s & HOLDS == 0 {
                        break;
                    }
                    if s & DEAD == 0 {
                        return false;
                    }
                    q = self.parts[a].parent;
                }
            }
        }
        // The release list (`0x5b8354..0x5b83de`); DDD leaves ctx+0x78/0x80/0x88 stale here.
        let fep = self.fep.clone();
        if let Some(def) = fep.bodies[self.body].parts.get(self.parts[pid].index).and_then(|d| d.as_ref()) {
            for &s in &def.lists[2] {
                let s = s as usize;
                if let Some(p) = self.parts[pid].params.get(s).copied().flatten() {
                    cmd::release(self, pid, s, p);
                    self.parts[pid].params[s] = None;
                }
            }
        }
        if let Some(q) = self.parts[pid].parent {
            if self.parts[q].alive {
                self.parts[q].children = self.parts[q].children.wrapping_sub(1);
            }
        }
        let pass = self.parts[pid].pass as usize;
        if let Some(k) = self.lists[pass].iter().position(|&p| p == pid) {
            self.lists[pass].remove(k);
        }
        self.parts[pid].alive = false;
        self.parts[pid].work = Vec::new();
        self.parts[pid].mega = Vec::new();
        self.parts[pid].locus = Vec::new();
        self.parts[pid].tail = Vec::new();
        self.parts[pid].wave = Vec::new();
        self.parts[pid].ribbon = Vec::new();
        self.parts[pid].lines = Vec::new();
        self.parts[pid].flash = Vec::new();
        self.free.push(pid);
        true
    }

    // --- the updates ---

    /// The pass-0 update (`0x5b53e0`; GUESS: as pass 1, no file has a pass-0 part), the pass-1
    /// update (`0x5b4700`) and the after-pass (`0x5b5e70`).
    fn update(&mut self) {
        for pass in 0..2 {
            // Phase 7: first-update slots of parts not started (`0x5b6061`).
            self.phase = if pass == 0 { 2 } else { 7 };
            self.walk(pass, |i, pid| {
                if i.parts[pid].state & STARTED != 0 {
                    return;
                }
                i.parts[pid].state |= STARTED;
                i.run_list(pid, 5, |i, pid, s, p, _| cmd::first(i, pid, s, p));
            });
            // Keys (`0x5b5040`).
            self.key_pass(pass);
            // Phase 8: update slots (`0x5b611a`).
            self.phase = if pass == 0 { 3 } else { 8 };
            self.walk(pass, |i, pid| i.run_list(pid, 0, cmd::update));
        }
        // Phase 0xc: the after-pass slots (`0x5b5e70`).
        self.phase = 0xc;
        self.walk(1, |i, pid| i.run_list(pid, 6, cmd::after));
    }

    /// Walk a pass list as `0x5b5f10` does: live parts only, parts appended meanwhile included.
    fn walk(&mut self, pass: usize, mut f: impl FnMut(&mut Instance, usize)) {
        let mut k = 0;
        while let Some(&pid) = self.lists[pass].get(k) {
            if self.parts[pid].state & DEAD == 0 {
                f(self, pid);
            }
            k += 1;
        }
    }

    /// Run slot list `list` of a part: `f(instance, part, slot, params, local time)`, local
    /// time = part time - the current entry's start (`0x5b6027`). Slots without a current block
    /// are skipped (DDD would pass null).
    fn run_list(&mut self, pid: usize, list: usize, mut f: impl FnMut(&mut Instance, usize, usize, usize, f32)) {
        let fep = self.fep.clone();
        let Some(def) = fep.bodies[self.body].parts.get(self.parts[pid].index).and_then(|d| d.as_ref()) else { return };
        for &s in &def.lists[list] {
            let s = s as usize;
            let p = &self.parts[pid];
            let Some(params) = p.params.get(s).copied().flatten() else { continue };
            let t = p.time - p.current[s].map_or(0.0, |e| e.start as f32);
            f(self, pid, s, params, t);
        }
    }

    /// The key pass (`0x5b5040`), skipped when this step's dt is 0: per live part, a pending
    /// wrap runs the keys up to loop end - 1, rewinds (`0x5b6680`), sets "looped" and moves the
    /// clock back into the loop: `(f - end) mod (end - start) + start + frac` (`0x5b50fa..
    /// 0x5b512b`); then the keys up to int(time).
    fn key_pass(&mut self, pass: usize) {
        if self.dt == 0.0 {
            return;
        }
        let mut k = 0;
        while let Some(&pid) = self.lists[pass].get(k) {
            k += 1;
            if self.parts[pid].state & DEAD != 0 {
                continue;
            }
            let mut f = self.parts[pid].time as i32;
            if self.parts[pid].state & WRAP != 0 {
                let (start, end) = self.def(pid).map_or((0, 0), |d| (d.loop_start as i32, d.loop_end as i32));
                self.keys(pid, end - 1);
                self.rewind(pid, end as f32);
                let p = &mut self.parts[pid];
                p.state = (p.state & 0xffef) | LOOPED;
                // DDD divides by end - start (`idiv`); an empty loop would fault there.
                let rem = if end != start { (f - end).wrapping_rem(end - start) } else { 0 };
                p.time = rem as f32 + start as f32 + (p.time - f as f32);
                f = p.time as i32;
            }
            self.keys(pid, f);
        }
    }

    /// The key runner (`0x5b5180`): while the next key list's frame <= f, each entry in order
    /// makes its block current and calls slot 1 (new block, old block, old span: the previous
    /// entry's span, or f when the slot had none, `0x5b5243`). `keys_run` counts the lists run
    /// in this call (`ctx+0xa0`, read by FecGenerate).
    fn keys(&mut self, pid: usize, f: i32) {
        self.keys_run = 0;
        let fep = self.fep.clone();
        let Some(def) = fep.bodies[self.body].parts.get(self.parts[pid].index).and_then(|d| d.as_ref()) else { return };
        loop {
            let k = self.parts[pid].next_key;
            if k < 0 {
                return;
            }
            let Some(list) = def.keys.get(k as usize) else {
                self.parts[pid].next_key = -1;
                return;
            };
            if list.frame as i32 > f {
                return;
            }
            for e in &list.entries {
                let s = e.slot as usize;
                let p = &mut self.parts[pid];
                let span = p.current[s].map_or(f, |c| c.span as i32);
                let old = p.params[s];
                let new = list.params(e);
                p.params[s] = Some(new);
                p.current[s] = Some(*e);
                cmd::key(self, pid, s, new, old, span as f32);
            }
            let p = &mut self.parts[pid];
            p.next_key += 1;
            self.keys_run += 1;
            if p.next_key as usize >= def.keys.len() {
                p.next_key = -1;
                return;
            }
        }
    }

    /// Loop rewind (`0x5b6680`), `t` = loop end. Next key = 0; then per loop-list slot: the key
    /// active at loop start is looked up **by class id** (`0x5b80c0`, see `loop_key`), becomes
    /// current, and slot 5 runs with (its block, loop start - its entry's start, old block, t -
    /// the old entry's start). The next key becomes the found list + 1 (maxed with the value
    /// so far), or -1 when nothing was found or no list follows (`0x5b6814..0x5b684c`).
    fn rewind(&mut self, pid: usize, t: f32) {
        let fep = self.fep.clone();
        let Some(def) = fep.bodies[self.body].parts.get(self.parts[pid].index).and_then(|d| d.as_ref()) else { return };
        self.parts[pid].next_key = 0;
        for &s in &def.lists[3] {
            let s = s as usize;
            let class = def.slots[s].id;
            let p = &self.parts[pid];
            let old = p.params[s];
            let t_old = t - p.current[s].map_or(0.0, |e| e.start as f32);
            let (found, idx, entry) = loop_key(def, class, def.loop_start as i32);
            let next = match found {
                Some(new) => {
                    let t_new = entry.map_or(0, |e| def.loop_start as i32 - e.start as i32) as f32;
                    let p = &mut self.parts[pid];
                    p.params[s] = Some(new);
                    p.current[s] = entry;
                    cmd::rewind(self, pid, s, new, t_new, old, t_old);
                    let n = idx as i32 + 1;
                    if def.keys.len() as i32 <= n {
                        -1
                    } else {
                        n.max(self.parts[pid].next_key as i32)
                    }
                }
                None => -1,
            };
            self.parts[pid].next_key = next as i16;
        }
    }
}

/// `0x5b80c0`: the key block active at frame `f` for a class. Over the key lists with frame <=
/// f, the first entry of each whose slot has class `class` (any slot of that class: the
/// README 7.2 quirk); the last such list wins. Not found: the init list's first entry of the
/// class (index 0; a block offset 0xffff counts as none). Returns the block, the list index and
/// DDD's "entry" out-value, which is the **last entry examined** (`r9`): the found one only
/// when the last list examined holds it. That is what DDD makes current, quirks included.
fn loop_key(def: &Part, class: u16, f: i32) -> (Option<usize>, usize, Option<Entry>) {
    let class_of = |e: &Entry| def.slots.get(e.slot as usize).map_or(0, |s| s.id);
    let mut found = None;
    let mut index = 0;
    let mut last = None;
    for (k, list) in def.keys.iter().enumerate() {
        if list.frame as i32 > f {
            break;
        }
        for e in &list.entries {
            last = Some(*e);
            if class_of(e) == class {
                found = Some(list.params(e));
                index = k;
                break;
            }
        }
    }
    if found.is_none() {
        index = 0;
        for e in &def.init.entries {
            last = Some(*e);
            if class_of(e) == class {
                if e.param != 0xffff {
                    found = Some(def.init.params(e));
                }
                break;
            }
        }
    }
    (found, index, last)
}

/// A live part, for drawing.
pub struct PartView<'a> {
    i: &'a Instance,
    pid: usize,
}

impl<'a> PartView<'a> {
    fn p(&self) -> &'a PartInst {
        &self.i.parts[self.pid]
    }

    /// Its index in the body's part table.
    pub fn index(&self) -> usize {
        self.p().index
    }
    /// The instance's id for it (stable while it lives; reused after).
    pub fn id(&self) -> usize {
        self.pid
    }
    /// The id of the generator part that spawned it.
    pub fn parent(&self) -> Option<usize> {
        self.p().parent
    }
    /// Its clock and lifetime, in frames.
    pub fn time(&self) -> f32 {
        self.p().time
    }
    pub fn lifetime(&self) -> u16 {
        self.p().lifetime
    }
    /// Has it looped (state bit 1)?
    pub fn looped(&self) -> bool {
        self.p().state & LOOPED != 0
    }
    pub fn matrix_type(&self) -> u8 {
        self.p().matrix_type
    }
    /// `+0x91`: FecLmatrix param byte 1 (or the parent's); low 3 bits 1 = camera-facing in
    /// FecMdl (INFERRED, README 8.2).
    pub fn lmatrix_flags(&self) -> u8 {
        self.p().b91
    }
    /// The local matrix (`+0x78`, FecLmatrix's S * R * T after its parent step), DDD's row
    /// convention stored as glam stores it (glam `M * v` = DDD `v * M`).
    pub fn local(&self) -> Option<Mat4> {
        self.p().local
    }
    /// The world matrix (`+0x80`).
    pub fn world(&self) -> Option<Mat4> {
        self.p().world
    }
    /// The frame it lives in: its owner's own frame (`+0x88`), else the instance matrix
    /// (README 7.3). INFERRED: the drawing applies it to the local matrix (commands.md 7.2).
    pub fn frame(&self) -> Mat4 {
        match self.p().owner {
            Some(o) => self.i.parts[o].frame.unwrap_or(Mat4::IDENTITY),
            None => self.i.effect,
        }
    }
    /// The output (first vec4 of the work) of slot `s`.
    pub fn output(&self, s: usize) -> Option<Vec4> {
        let def = self.i.def(self.pid)?;
        Some(self.p().rd4(def.slots.get(s)?.work_offset()))
    }
    /// A FecLocus_tail_strategy slot's points (all N, newest first, in its sample space) and how
    /// many are reached.
    pub fn locus(&self, s: usize) -> Option<(Vec<Vec3>, usize)> {
        cmd::locus_points(self.i, self.pid, s)
    }
    /// The output of the first slot of a Col* class: RGBA 0..255 (128 = "full", GUESS).
    pub fn colour(&self) -> Option<Vec4> {
        self.first_of(cmd::is_colour)
    }
    /// The output of the first slot of a Scl* class: the scale factor (percent x 0.01), xyz.
    pub fn scale(&self) -> Option<Vec3> {
        self.first_of(cmd::is_scale).map(|v| v.xyz())
    }
    /// The output of the first slot of a Pos* class: the position before the y / z negation.
    pub fn position(&self) -> Option<Vec3> {
        self.first_of(cmd::is_position).map(|v| v.xyz())
    }
    fn first_of(&self, f: fn(u16) -> bool) -> Option<Vec4> {
        let def = self.i.def(self.pid)?;
        let s = def.slots.iter().position(|s| f(s.id))?;
        self.output(s)
    }

    /// Its draw commands: the part's draw list (`+0x2c`), in order (README 6.4: use the
    /// file's list; some parts leave a drawing class out).
    pub fn draws(&self) -> impl Iterator<Item = DrawCmd<'a>> + 'a {
        let i = self.i;
        let pid = self.pid;
        let p = &i.parts[pid];
        let def = i.def(pid);
        def.into_iter().flat_map(move |def| {
            def.lists[1].iter().filter_map(move |&s| {
                let s = s as usize;
                let slot = def.slots.get(s)?;
                let at = p.params.get(s).copied().flatten()?;
                let params = i.fep.params(at, slot.id);
                let w = slot.work_offset();
                let ws = sig::work_size(slot.id).max(0x40);
                let colour = slot
                    .inputs
                    .iter()
                    .find(|inp| def.slots.get(inp.slot as usize).is_some_and(|q| cmd::is_colour(q.id)))
                    .map(|inp| p.rd4(inp.work as usize * 2));
                Some(DrawCmd {
                    slot: s,
                    class: slot.id,
                    name: sig::name(slot.id),
                    status: cmd::class_status(slot.id),
                    params,
                    params_offset: at,
                    resource: match slot.id {
                        152..=155 => Some(0x0c),
                        80 => Some(0x2c),
                        _ => None,
                    }
                    .map(|o| i32::from_le_bytes(params.get(o..o + 4).and_then(|b| b.try_into().ok()).unwrap_or([0xff; 4]))),
                    local_time: p.time - p.current[s].map_or(0.0, |e| e.start as f32),
                    work: p.work.get(w..(w + ws).min(p.work.len())).unwrap_or(&[]),
                    colour,
                    sprites: if slot.id == 153 { cmd::mega_sprites(i, pid, s, at) } else { Vec::new() },
                    tail: if slot.id == 80 { cmd::tail_draw(i, pid, s, at) } else { None },
                    ribbon: if slot.id == 105 { cmd::ribbon_draw(i, pid, s, at) } else { None },
                    lines: if matches!(slot.id, 100 | 104) { cmd::lines_draw(i, pid, s, at) } else { None },
                    flash: if slot.id == 107 { cmd::flash_draw(i, pid, s, at) } else { None },
                })
            })
        })
    }
}

/// One draw command of a live part: the data DDD's draw slot gets (the drawing is the caller's;
/// the sprite draws are in docs/fep/draw-sprites.md), and FecMegaanm_plus's sprites as its draw
/// evaluates them.
#[derive(Debug, Clone)]
pub struct DrawCmd<'a> {
    /// Slot index in the part.
    pub slot: usize,
    /// Command id (152 Anm_plus, 153 Megaanm_plus, 154 Mdl_plus, 155 Mdlts_plus, ...).
    pub class: u16,
    pub name: &'static str,
    pub status: ClassStatus,
    /// The current parameter block (`sig::param_size` bytes; the `_plus` classes: base layout +
    /// a trailing field, README 3) and its file offset.
    pub params: &'a [u8],
    pub params_offset: usize,
    /// Param `+0x0c` (s32) for the four drawing classes: the section-3 (MDL) index for
    /// Mdl_plus / Mdlts_plus, section-4 (ANM) index for Anm_plus / Megaanm_plus (README 8.6);
    /// FecShapetail: its ANM index `+0x2c`; `None` for other classes.
    pub resource: Option<i32>,
    /// Part time - the current entry's start (as the draw slot gets it).
    pub local_time: f32,
    /// The command's work slice (at least 0x40 bytes when the part has them): Anm_plus `+0`
    /// f32 clock, `+4` u16 frame (0xffff = ended), `+6` u8 flip; Mdlts_plus three vec2 at
    /// `+0x08` value, `+0x10` velocity, `+0x18` acceleration (`+0x00` the value at init).
    pub work: &'a [u8],
    /// The output of its first Col* input (RGBA 0..255), if it has one.
    pub colour: Option<Vec4>,
    /// FecMegaanm_plus: its live sprites in draw order (oldest first, age < life), evaluated as
    /// its draw does (`0x58e010`); empty for other classes, without the buffer or the ANM.
    pub sprites: Vec<Sprite>,
    /// FecShapetail: what its draw (`0x5a2850`) would draw this frame; `None` for other classes,
    /// and when it draws nothing (no ANM, animation over, Lmatrix byte 1 not 0 / 1).
    pub tail: Option<TailDraw>,
    /// FecBindlocus_polygon: the strip its draw (`0x59deb0`) would draw this frame; `None` for
    /// other classes and when it draws nothing (textured without its texture).
    pub ribbon: Option<RibbonDraw>,
    /// FecEllipse_line_particle / FecRadialg_line_particle: the line strips their draws
    /// (`0x560130`, `0x563c80`) would draw; `None` for other classes and without a pool.
    pub lines: Option<LineDraw>,
    /// FecFlash: the screen rectangle its draw (`0x596da0`) would fill; `None` for other classes.
    pub flash: Option<FlashDraw>,
}

/// A FecFlash draw (aura.md 5.3): one untextured quad over a rectangle of DDD's virtual screen
/// (`cmd::SCREEN`, 400 x 240, origin top-left INFERRED), drawn without the depth test (INFERRED)
/// and with an identity world. DDD's draw: colour `K = clamp(2 x colour x element colour, 0,
/// 255)` per component, through the untextured preset `+0xa4` (INFERRED: rgb = K / 255, alpha =
/// K.a / 255 x 128 / 255); blend by `blend` through `0x7ed8e0`; skipped when the element
/// instance's `+0x1b6` is set (not modelled).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlashDraw {
    /// `x0, y0, x1, y1` in virtual pixels as slot 2 left them (the whole screen: 0, 0, 400, 240).
    pub rect: [i16; 4],
    /// `+0x19` = 0: the whole screen (the rect fields unused).
    pub full_screen: bool,
    /// Input 0's output vec4 (FSH0 part 10: its Colac), 0..255, before the element's colour.
    pub colour: Vec4,
    /// `+0x18`: an index for `0x7ed8e0` (0 -> 1, 1 -> 2 additive, 2 -> 3, 3 -> 1).
    pub blend: u8,
    /// `v0 + rate x t` (`+0x10`, `+0x14`): computed, never read by DDD's draw.
    pub value: f32,
}

/// A FecBindlocus_polygon draw (tails.md 2.5, 2.7): one triangle strip `A0 B0 A1 B1 ...` (2 x
/// `count` vertices, point 0 = the newest sample), in the frame's space (DDD: world, drawn with
/// an identity world matrix). Colour: vertex bytes x the texel, **128 = 1.0** (textured preset
/// `+0xdc`, render-state.md 3), before the element's colour; the untextured preset would take
/// RGB 255 = 1.0, alpha 128 = 1.0 as `LineDraw`.
#[derive(Debug, Clone)]
pub struct RibbonDraw {
    pub vertices: Vec<RibbonVertex>,
    /// Section-2 index of the texture (`+0x2c`) when textured (`+0x48`); `None` = untextured.
    pub texture: Option<i32>,
    /// `+0x51`: an index for `0x7ed8e0` (0 -> 1, 1 -> 2 (additive weighted by alpha), 2 -> 3,
    /// 3 -> 1).
    pub blend: u8,
    /// `+0x4b`: the sampler flag (1 = as every effect draw).
    pub sampler: bool,
    /// `+0x20` when the depth offset `+0x55` is on (raw: y, z as stored; tails.md 2.7).
    pub depth: Option<Vec3>,
}

/// One ribbon vertex: the 24-byte record of `0x59d360` (`{f32 u, v, u32 rgba, f32 xyz}`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RibbonVertex {
    pub position: Vec3,
    /// Texture coordinate (1.0 = the texture's width / height).
    pub uv: glam::Vec2,
    /// R, G, B, A bytes (the head fade already in A).
    pub colour: [u8; 4],
}

/// A line-particle draw (tails.md 4.4, 5.4): one line strip per live particle, in slot order,
/// head (newest) first. Positions are in the space `matrix` takes to the part's frame through
/// DDD's world-matrix helper `0x5b2190` (draw-sprites 4.4, with f = 1) by Lmatrix byte 1
/// (`facing`): FecEllipse_line_particle `matrix = diag(axes) x local` (its vertices are
/// part-local), FecRadialg_line_particle `matrix = local` (DDD transforms them on the CPU, the
/// same product). Colour: untextured preset `+0xf8` (B = 0x201, render-state.md 3): **RGB byte
/// 255 = 1.0, alpha byte 128 = 1.0**; before the element's colour.
#[derive(Debug, Clone)]
pub struct LineDraw {
    pub strips: Vec<Vec<LineVertex>>,
    /// DDD's matrix (glam form; DDD `A x B` is glam `B * A`).
    pub matrix: Mat4,
    /// Lmatrix (input 1) byte 1.
    pub facing: u8,
    /// Blend byte (`+0x4c` / `+0x44`): an index for `0x7ed8e0` as `RibbonDraw::blend`.
    pub blend: u8,
}

/// One line vertex (format 2, `{u32 rgba, f32 xyz}`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineVertex {
    pub position: Vec3,
    pub colour: [u8; 4],
}

/// A FecShapetail draw (tails.md 1.7): one ANM sprite (key `anm_frame`) per reached Locus point,
/// head first. Each sprite's matrix (DDD rows) is `diag(k) * RotZ(angle)` translated to the
/// point, `k = scale x ANM size (+0x38) x 0.1`; then:
/// - `facing` false (Lmatrix byte 1 = 0): times D, D = the part's frame when not `world`
///   (INFERRED for `0x5b2670`, not read), identity when `world`;
/// - `facing` true (byte 1 = 1): the point taken through D, `k` times the instance scale, the
///   3x3 times the camera's (screen-aligned).
///
/// Colour: the point's x the element's colour (/128), clamped 0..255, then the ANM quad colour x
/// that >> 7. Blend: `blend` 4 = each quad's own code, else the `0x7ed8e0` index (1..3).
#[derive(Debug, Clone)]
pub struct TailDraw {
    pub anm_frame: u16,
    /// Locus space `+0x26`: false = the points are in the part's frame space (through its local
    /// matrix), true = world (local x frame).
    pub world: bool,
    pub facing: bool,
    /// Shapetail `+0x48`: 0 -> 4, 1..3 as is (an index for `0x7ed8e0`).
    pub blend: u8,
    /// `+0x20` when orient `+0x47` is on (tails.md 1.8, not decoded: not applied).
    pub orient: Option<Vec3>,
    /// Points reached (Locus `+0xb0`); `points` has that many.
    pub reached: usize,
    pub points: Vec<TailPoint>,
}

/// One Shapetail sprite.
#[derive(Debug, Clone, Copy)]
pub struct TailPoint {
    /// The Locus point (engine space, y / z as DDD has them; see `TailDraw::world`).
    pub position: Vec3,
    /// Size factor: percent lerped head -> tail x 0.01 x the point's random size.
    pub scale: f32,
    /// Degrees about the sprite's z (RotZ `0x565b50`).
    pub angle: f32,
    /// RGBA 0..255 (128 = 1.0): Shapetail's colour lerped head -> tail x its colour input / 128,
    /// clamped; before the element's colour.
    pub colour: Vec4,
}

/// One FecMegaanm_plus sprite as the draw (`0x58e010`, draw-sprites.md 5.4) has it before the
/// ANM-size and element factors and the billboard. The draw's matrix (DDD row vectors) is
/// `diag(scale x ANM size x 0.001 x element scale) * Flip(flip) * RotZ(-angle)`, translated to
/// `position` (taken through the local matrix then the frame unless `world`), then its 3x3
/// times the inverse view rotation (screen-aligned).
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    /// The ANM key to show (record `+0x7c`, set by the last update).
    pub anm_frame: u16,
    /// Flip bits (`+0x7e`): bit 0 RotY 180, bit 1 RotX 180 (`0x5bce80`).
    pub flip: u8,
    /// `pos + vel age + acc age^2 / 2`: part-local engine space (y, z already negated), or world
    /// when `world` (space `+0xfb` 2, 6 or 7).
    pub position: Vec3,
    pub world: bool,
    /// `scale + vel age + acc age^2 / 2`, percent (z = 100).
    pub scale: Vec3,
    /// Screen angle in degrees, `rot + spin age + spin_acc age^2 / 2`; the draw turns by -angle.
    pub angle: f32,
    /// RGBA (128 = 1.0): colour + velocity x age, alpha x the fade. Not clamped: DDD clamps to
    /// 0..255 after multiplying the element's colour.
    pub colour: Vec4,
    /// Age in frames.
    pub age: f32,
}

impl DrawCmd<'_> {
    /// The blend index (the `_plus` byte): FecAnm_plus `+0x1f`, FecMegaanm_plus `+0x115`.
    pub fn blend_index(&self) -> Option<u8> {
        match self.class {
            152 => self.params.get(0x1f).copied(),
            153 => self.params.get(0x115).copied(),
            _ => None,
        }
    }
    /// FecShapetail: its `TailDraw::blend`.
    pub fn tail_blend(&self) -> Option<u8> {
        self.tail.as_ref().map(|t| t.blend)
    }
    /// FecMegaanm_plus's depth offset `+0xec` (view z, subtracted: negative = toward the camera),
    /// when on (`+0x10e` and not 0, `0x58e13f`).
    pub fn mega_depth(&self) -> Option<f32> {
        let d = f32::from_le_bytes(self.params.get(0xec..0xf0)?.try_into().ok()?);
        (self.class == 153 && self.params.get(0x10e).is_some_and(|&b| b != 0) && d != 0.0).then_some(d)
    }
    /// FecAnm_plus: the current ANM key (work `+4`; `None` = ended, 0xffff).
    pub fn anm_frame(&self) -> Option<u16> {
        let f = u16::from_le_bytes(self.work.get(4..6)?.try_into().ok()?);
        (f != 0xffff).then_some(f)
    }
    /// FecMdlts_plus: the vec2 value (work `+0x08`; GUESS a texture scroll).
    pub fn mdlts_value(&self) -> Option<glam::Vec2> {
        let r = |o: usize| self.work.get(o..o + 4).and_then(|b| b.try_into().ok()).map(f32::from_le_bytes);
        Some(glam::Vec2::new(r(8)?, r(12)?))
    }
}
