//! The command classes: what each vtable slot does (docs/fep/README.md section 8, commands.md
//! section 6), read from the exe at the RVAs given.
//!
//! A command keeps its state in its slice of the part's work area (the first vec4 its output);
//! the others read it through their inputs (README 7.3). Slots: 0 init, 1 key (a new parameter
//! block takes over; `t` = the old entry's span), 2 update (`t` = local time), 4 release,
//! 5 loop rewind, 6 first update, 7 after-pass. Units: rotation in degrees, scale in percent
//! (output x0.01), colour 0..255 (clamped).
//!
//! Implemented: Pos/Posmv/Posac, Rot/Rotmv/Rotac, Scl/Sclmv/Sclac, Col/Colmv/Colac (1-9, 15-17),
//! FecLmatrix (14), FecGenerate (18), FecGravity (19), FecRndpos/rot/scl/col (22-25),
//! FecColorblink_parameter / FecColorblink (61, 62), FecLocus_tail_strategy (68),
//! FecNull_routegenerator (79), FecWave_bindlocus_strategy (106), FecGenerateanchorvertex (150),
//! FecGenerateanchor (151). Drawing classes keep their state for the drawer: FecShapetail (80, its
//! clock and per-point sizes / rotations; the sprites its draw evaluates), FecRadialg_line_particle
//! (100) and FecEllipse_line_particle (104) (their particle pools; the line strips their draws
//! evaluate), FecBindlocus_polygon (105, its UV scroll and spline; the strip it rebuilds), FecFlash (107,
//! its screen rectangle), FecAnm_plus (152, its clock), FecMegaanm_plus
//! (153, its sprite buffer: emission, ageing, the per-sprite values its draw evaluates),
//! FecMdl_plus (154), FecMdlts_plus (155, its vec2). Every other class is a stub: init
//! succeeds, nothing else happens (`class_status`).

use glam::{Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::parse::{Fep, Slot};
use crate::res;
use crate::rng::{self, Rng};
use crate::run::{Instance, SpawnInfo, LOOPED};

/// How far a class is implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassStatus {
    /// Every slot DDD overrides is implemented.
    Implemented,
    /// A drawing class: its state is kept and exposed (`DrawCmd`) for a drawer, which draws.
    DrawData,
    /// Not implemented: init succeeds, nothing else happens.
    Stub,
}

/// How far the class of a command id is implemented here.
pub fn class_status(id: u16) -> ClassStatus {
    match id {
        1..=9 | 14..=19 | 22..=25 | 61 | 62 | 68 | 79 | 106 | 150 | 151 => ClassStatus::Implemented,
        80 | 100 | 104 | 105 | 107 | 152..=155 => ClassStatus::DrawData,
        _ => ClassStatus::Stub,
    }
}

// f32 constants, bit for bit.
/// `[0x7ecf84]` 256/360 and `[0x7ecf80]` 2*pi/256: degrees to radians (`0x565a3d`).
const DEG_TO_256: f32 = 256.0 / 360.0;
const B256_TO_RAD: f32 = std::f32::consts::TAU / 256.0;
/// `[0x867d08]`: scale percent.
const PERCENT: f32 = 0.01;
/// `[0x7ed324]`: gravity per unit.
const GRAVITY: f32 = 0.098;

// Work layout of the Pos/Rot/Scl/Col families (README 8.1).
const OUT: usize = 0x00;
const BASE: usize = 0x10;
const VEL: usize = 0x20;
const ACC: usize = 0x30;

// --- parameter reads (file offsets; 0 past the end) ---

fn pu8(f: &Fep, at: usize) -> u8 {
    f.data.get(at).copied().unwrap_or(0)
}
fn pu16(f: &Fep, at: usize) -> u16 {
    u16::from_le_bytes([pu8(f, at), pu8(f, at + 1)])
}
fn pi16(f: &Fep, at: usize) -> i16 {
    pu16(f, at) as i16
}
fn pu32(f: &Fep, at: usize) -> u32 {
    u32::from_le_bytes([pu8(f, at), pu8(f, at + 1), pu8(f, at + 2), pu8(f, at + 3)])
}
fn pf(f: &Fep, at: usize) -> f32 {
    f32::from_bits(pu32(f, at))
}
fn pv4(f: &Fep, at: usize) -> Vec4 {
    Vec4::new(pf(f, at), pf(f, at + 4), pf(f, at + 8), pf(f, at + 12))
}
fn pv2(f: &Fep, at: usize) -> Vec2 {
    Vec2::new(pf(f, at), pf(f, at + 4))
}

/// Byte offset of a slot's input `k` in the work area (0 when it has none, as DDD reads it).
fn input_work(slot: &Slot, k: usize) -> usize {
    slot.inputs.get(k).map_or(0, |i| i.work as usize * 2)
}

/// The slot descriptor of slot `s` of a part instance, cloned out of the shared FEP.
fn slot_of(i: &Instance, pid: usize, s: usize) -> Option<Slot> {
    i.def(pid).and_then(|d| d.slots.get(s)).cloned()
}

// --- the Pos / Rot / Scl / Col families ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fam {
    Pos,
    Rot,
    Scl,
    Col,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    Value,
    Mv,
    Ac,
}

fn family(id: u16) -> Option<(Fam, Form)> {
    use Fam::*;
    use Form::*;
    Some(match id {
        1 => (Pos, Value),
        2 => (Pos, Mv),
        3 => (Pos, Ac),
        4 => (Rot, Value),
        5 => (Rot, Mv),
        6 => (Rot, Ac),
        7 => (Scl, Value),
        8 => (Scl, Mv),
        9 => (Scl, Ac),
        15 => (Col, Value),
        16 => (Col, Mv),
        17 => (Col, Ac),
        _ => return None,
    })
}

/// Components a family's motion covers: vec3, colour vec4.
fn comps(f: Fam) -> usize {
    if f == Fam::Col {
        4
    } else {
        3
    }
}

/// Clamp to 0..255 as `0x57f979..0x57f9f8` (`[0x78e78c]`).
fn clamp255(v: Vec4) -> Vec4 {
    let c = |x: f32| if 0.0 > x { 0.0 } else if 255.0 < x { 255.0 } else { x };
    Vec4::new(c(v.x), c(v.y), c(v.z), c(v.w))
}

/// Write a family's output from a value: Pos/Rot the vec4 as is, Scl xyz x0.01 (w kept), Col
/// clamped.
fn output(i: &mut Instance, pid: usize, at: usize, fam: Fam, v: Vec4) {
    let p = &mut i.parts[pid];
    match fam {
        Fam::Pos | Fam::Rot => p.wr4(at + OUT, v),
        Fam::Scl => p.wr3(at + OUT, v.xyz() * PERCENT),
        Fam::Col => p.wr4(at + OUT, clamp255(v)),
    }
}

/// Spawn-info bit of a family's value (README 7.3): position 0, rotation 3, scale 6, colour 9.
fn spawn_bit(f: Fam) -> usize {
    match f {
        Fam::Pos => 0,
        Fam::Rot => 3,
        Fam::Scl => 6,
        Fam::Col => 9,
    }
}

/// Slot 0: `0x583570` Pos, `0x583290` Posmv, `0x582e10` Posac, `0x583d00` Rot, `0x583a80` Rotmv,
/// `0x583760` Rotac, `0x584320` Scl, `0x584120` Sclmv, `0x583e60` Sclac, `0x57f950` Col,
/// `0x57f670` Colmv, `0x57f2f0` Colac. Vec3 families take w = 0. The spawn info replaces the
/// value / velocity / acceleration, except FecPosac, which adds them and sets w = 1
/// (`0x582e6f..0x582f69`).
fn fam_init(i: &mut Instance, pid: usize, at: usize, p: usize, fam: Fam, form: Form) {
    let fep = i.fep.clone();
    let rd = |o: usize| if fam == Fam::Col { pv4(&fep, p + o) } else { pv4(&fep, p + o).truncate().extend(0.0) };
    let mut v = [rd(0), rd(0x10), rd(0x20)];
    let n = match form {
        Form::Value => 1,
        Form::Mv => 2,
        Form::Ac => 3,
    };
    if let Some(si) = i.spawn_info {
        for (k, val) in v.iter_mut().enumerate().take(n) {
            if fam == Fam::Pos && form == Form::Ac {
                if let Some(s) = si.get(k) {
                    *val = (val.truncate() + s.truncate()).extend(1.0);
                }
            } else if let Some(s) = si.get(spawn_bit(fam) + k) {
                *val = s;
            }
        }
    }
    let part = &mut i.parts[pid];
    part.wr4(at + BASE, v[0]);
    if n > 1 {
        part.wr4(at + VEL, v[1]);
    }
    if n > 2 {
        part.wr4(at + ACC, v[2]);
    }
    output(i, pid, at, fam, v[0]);
}

/// Slot 1. Value forms (`0x583510`...): base.xyz += new - old (FecCol too: its w is not rebased,
/// `0x57f880`). mv forms (`0x583150`): base = (base + v t) - (old.p + old.v t) + new.p, v' = v -
/// old.v + new.v. ac forms (`0x5afe70`, vec4 `0x5b0090`): the same with acceleration (README
/// 8.1). Output: refreshed from the base, except FecPosac and FecColac (they jump straight to the
/// rebase, `0x582d60`, `0x57f1d0`).
fn fam_key(i: &mut Instance, pid: usize, at: usize, fam: Fam, form: Form, new: usize, old: Option<usize>, t: f32) {
    let Some(old) = old else { return };
    let fep = i.fep.clone();
    let part = &mut i.parts[pid];
    let (mut b, mut v, mut a) = (part.rd4(at + BASE), part.rd4(at + VEL), part.rd4(at + ACC));
    let (np, nv, na) = (pv4(&fep, new), pv4(&fep, new + 0x10), pv4(&fep, new + 0x20));
    let (op, ov, oa) = (pv4(&fep, old), pv4(&fep, old + 0x10), pv4(&fep, old + 0x20));
    let n = comps(fam);
    match form {
        Form::Value => {
            for k in 0..3 {
                b[k] = b[k] - op[k] + np[k];
            }
        }
        Form::Mv => {
            for k in 0..n {
                b[k] = (b[k] + v[k] * t) - (op[k] + ov[k] * t) + np[k];
                v[k] = v[k] - ov[k] + nv[k];
            }
        }
        Form::Ac => {
            let tt = t * t;
            for k in 0..n {
                let cur = a[k] * 0.5 * tt + v[k] * t + b[k];
                let was = oa[k] * 0.5 * tt + t * ov[k] + op[k];
                let vel = (a[k] * t + v[k]) - (t * oa[k] + ov[k]) + nv[k];
                b[k] = cur - was + np[k];
                v[k] = vel;
                a[k] = a[k] - oa[k] + na[k];
            }
        }
    }
    part.wr4(at + BASE, b);
    if form != Form::Value {
        part.wr4(at + VEL, v);
    }
    if form == Form::Ac {
        part.wr4(at + ACC, a);
    }
    if !(form == Form::Ac && matches!(fam, Fam::Pos | Fam::Col)) {
        output(i, pid, at, fam, b);
    }
}

/// Slot 2 of the mv / ac forms: out = base + v t (+ a t^2 / 2) (`0x583240`, `0x582d80` with 0.5
/// at `[0x867d7c]`, `0x5836e0`, `0x583dd0`, `0x57f1f0`...). Pos and Col write all four
/// components, Rot and Scl xyz.
fn fam_update(i: &mut Instance, pid: usize, at: usize, fam: Fam, form: Form, t: f32) {
    let part = &mut i.parts[pid];
    let (b, v, a) = (part.rd4(at + BASE), part.rd4(at + VEL), part.rd4(at + ACC));
    let val = match form {
        Form::Value => return,
        Form::Mv => b + v * t,
        Form::Ac => a * (0.5 * t * t) + v * t + b,
    };
    match fam {
        Fam::Rot => part.wr3(at + OUT, val.xyz()),
        _ => output(i, pid, at, fam, val),
    }
}

/// Keep a rotation in [-360, 360] (`0x583b94..0x583c69`). Absurd values are reduced first so
/// the loop ends (DDD would spin forever on them).
fn wrap360(x: f32) -> f32 {
    if !x.is_finite() {
        return x;
    }
    let mut x = if x.abs() > 1.0e6 { x % 360.0 } else { x };
    while x > 360.0 {
        x -= 360.0;
    }
    while -360.0 > x {
        x += 360.0;
    }
    x
}

/// Slot 5 (loop rewind; `t_new` = local time at loop start, `t_old` = loop end - old entry's
/// start). Value forms (`0x5835c0`...): base = out (FecScl: out x100, `0x5843a0`). mv forms
/// (`0x583300`, `0x583af0`): v' = v - old.v + new.v, base = base + v t_old - v' t_new. ac forms:
/// `0x5b04e0` (vec4 `0x5b0720`), the part continues from where it was (README 8.1). FecPosmv and
/// FecPosac first test new byte `+0x0c` == 1 with t_new == 0: then they restart from the block
/// (`0x583304`, `0x582f9f`; 0 in every file). Rot wraps its base into [-360, 360]. Then slot 2.
/// FecSclmv / FecColmv (`0x5841c0`, `0x57f740`; unused by the Flowmotion files) are taken to
/// work like FecRotmv without the wrap (INFERRED, not read).
fn fam_rewind(i: &mut Instance, pid: usize, at: usize, fam: Fam, form: Form, new: usize, t_new: f32, old: Option<usize>, t_old: f32) {
    let fep = i.fep.clone();
    if form == Form::Value {
        let part = &mut i.parts[pid];
        let out = part.rd4(at + OUT);
        if fam == Fam::Scl {
            part.wr3(at + BASE, out.xyz() * 100.0);
        } else {
            part.wr4(at + BASE, out);
        }
        return;
    }
    if fam == Fam::Pos && pu8(&fep, new + 0x0c) == 1 && t_new == 0.0 {
        let part = &mut i.parts[pid];
        let b = pv4(&fep, new).truncate().extend(0.0);
        part.wr4(at + BASE, b);
        part.wr4(at + VEL, pv4(&fep, new + 0x10).truncate().extend(0.0));
        if form == Form::Ac {
            part.wr4(at + ACC, pv4(&fep, new + 0x20).truncate().extend(0.0));
        }
        part.wr4(at + OUT, b);
        return;
    }
    let Some(old) = old else { return };
    let part = &mut i.parts[pid];
    let (mut b, mut v, mut a) = (part.rd4(at + BASE), part.rd4(at + VEL), part.rd4(at + ACC));
    let (nv, na) = (pv4(&fep, new + 0x10), pv4(&fep, new + 0x20));
    let (ov, oa) = (pv4(&fep, old + 0x10), pv4(&fep, old + 0x20));
    for k in 0..comps(fam) {
        if form == Form::Mv {
            let v2 = v[k] - ov[k] + nv[k];
            b[k] = (v[k] * t_old + b[k]) - v2 * t_new;
            v[k] = v2;
        } else {
            let v2 = (a[k] * t_old + v[k]) - (t_old * oa[k] + ov[k]) + nv[k];
            let a2 = a[k] - oa[k] + na[k];
            let cur = a[k] * 0.5 * (t_old * t_old) + v[k] * t_old + b[k];
            b[k] = cur - (a2 * 0.5 * (t_new * t_new) + v2 * t_new);
            v[k] = v2;
            a[k] = a2;
        }
    }
    if fam == Fam::Rot {
        for k in 0..3 {
            b[k] = wrap360(b[k]);
        }
    }
    part.wr4(at + BASE, b);
    part.wr4(at + VEL, v);
    if form == Form::Ac {
        part.wr4(at + ACC, a);
    }
    fam_update(i, pid, at, fam, form, t_new);
}

/// Slot 6 of Pos/Posmv/Posac (`0x5835e0`, `0x583440`, `0x583080`): a root part of matrix type 2
/// has its base (y, z negated) moved by the effect matrix once, through `0x5b21a0` (not read;
/// INFERRED: a point transform). No Flowmotion part has type 2.
fn pos_first(i: &mut Instance, pid: usize, at: usize) {
    if i.parts[pid].parent.is_some() || i.parts[pid].matrix_type != 2 {
        return;
    }
    let effect = i.effect;
    let part = &mut i.parts[pid];
    let b = part.rd4(at + BASE);
    let q = effect.transform_point3(Vec3::new(b.x, -b.y, -b.z));
    let nb = Vec4::new(q.x, -q.y, -q.z, b.w);
    part.wr4(at + BASE, nb);
    part.wr4(at + OUT, nb);
}

// --- FecLmatrix (14) ---

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

/// A rotation matrix of DDD (`0x565a20` RotX, `0x55f3e0` RotY, `0x565b50` RotZ), in glam's
/// column form: DDD's rows are glam's columns. Degrees -> radians as `0x565a3d`: x 256/360 x
/// 2pi/256; `sinf` / `cosf` (imports `0x771c76` / `0x771c64`).
fn rot(axis: Axis, deg: f32) -> Mat4 {
    let a = deg * DEG_TO_256 * B256_TO_RAD;
    let (s, c) = (a.sin(), a.cos());
    match axis {
        Axis::X => Mat4::from_cols(Vec4::X, Vec4::new(0.0, c, s, 0.0), Vec4::new(0.0, -s, c, 0.0), Vec4::W),
        Axis::Y => Mat4::from_cols(Vec4::new(c, 0.0, -s, 0.0), Vec4::Y, Vec4::new(s, 0.0, c, 0.0), Vec4::W),
        Axis::Z => Mat4::from_cols(Vec4::new(c, s, 0.0, 0.0), Vec4::new(-s, c, 0.0, 0.0), Vec4::Z, Vec4::W),
    }
}

/// The local matrix (`0x582990`, jump table `0x582afc`): DDD's `S * Ra * Rb * Rc` (row vectors,
/// `0x63ac90` multiplies `out = src * R`), translation row = position, w 1. Orders 0..5 = ZYX,
/// ZXY, YZX, YXZ, XZY, XYZ; a larger order leaves S alone, without rotation or translation.
/// Returned in glam's column form (`Rc * Rb * Ra * S`).
pub(crate) fn local_matrix(scl: Vec3, r: Vec3, pos: Vec3, order: u8) -> Mat4 {
    let s = Mat4::from_diagonal(scl.extend(1.0));
    use Axis::*;
    let (a, b, c) = match order {
        0 => (Z, Y, X),
        1 => (Z, X, Y),
        2 => (Y, Z, X),
        3 => (Y, X, Z),
        4 => (X, Z, Y),
        5 => (X, Y, Z),
        _ => return s,
    };
    let ang = |ax: Axis| match ax {
        X => r.x,
        Y => r.y,
        Z => r.z,
    };
    let mut m = rot(c, ang(c)) * (rot(b, ang(b)) * (rot(a, ang(a)) * s));
    m.w_axis = pos.extend(1.0);
    m
}

/// Slots 0 (`0x5825c0`) and 2 (`0x5821b0`). Inputs: 0 = Pos*, 1 = Scl*, 2 = Rot* (`0x58260f..
/// 0x582694`); y and z of position and rotation negated (`0x5826a1..0x5826d4`). Slot 2 also
/// copies param byte 1 to the instance (`+0x91`, `0x5821e9`). Then the world step (`0x582770`
/// -> `0x5b10e0`) with byte 0 as its mode.
/// TODO: byte 2 == 1 (orient along the Pos command's velocity, `0x5822fd..0x582493`) is not
/// implemented: 0 in every file.
fn lmatrix(i: &mut Instance, pid: usize, s: usize, p: usize, update: bool) {
    let fep = i.fep.clone();
    let Some(slot) = slot_of(i, pid, s) else { return };
    let part = &mut i.parts[pid];
    let pos = part.rd4(input_work(&slot, 0));
    let scl = part.rd4(input_work(&slot, 1));
    let r = part.rd4(input_work(&slot, 2));
    if update {
        part.b91 = pu8(&fep, p + 1);
    }
    let m = local_matrix(scl.xyz(), Vec3::new(r.x, -r.y, -r.z), Vec3::new(pos.x, -pos.y, -pos.z), pu8(&fep, p + 3));
    world(i, pid, m, pu8(&fep, p));
}

/// Rows 0..2 of a matrix normalised as vec4 (`0x5b2d00` -> `0x598ae0`; zero stays zero).
fn normalised(m: Mat4) -> Mat4 {
    let n = |v: Vec4| if v.length_squared() == 0.0 { Vec4::ZERO } else { v / v.length() };
    Mat4::from_cols(n(m.x_axis), n(m.y_axis), n(m.z_axis), m.w_axis)
}

/// DDD `A3 * B3` on the 3x3 parts (`0x5b2de0` -> `0x63a950`, order INFERRED from its callers'
/// argument pattern), w components and translation kept.
fn mul3(a: Mat4, b: Mat4) -> Mat4 {
    let r = Mat3::from_mat4(b) * Mat3::from_mat4(a);
    Mat4::from_cols(r.x_axis.extend(a.x_axis.w), r.y_axis.extend(a.y_axis.w), r.z_axis.extend(a.z_axis.w), a.w_axis)
}

/// The world step `0x5b10e0` (mode = Lmatrix byte 0; it equals the part's matrix type in every
/// Flowmotion block). World = local, then with a parent that has a local matrix:
/// - mode 0: the translations of both are moved by the parent's local matrix (`0x5b13e5..
///   0x5b1488`);
/// - mode 3: local = local x parent local, world = world x parent world (`0x5b131e..0x5b1359`);
/// - mode 4: the 3x3 parts are multiplied by the parent's with its axes normalised (no scale),
///   the translations moved by the parent's matrices (`0x5b1202..0x5b12e3`);
/// - modes 1, 2, >= 5: nothing.
///
/// In modes 0, 3, 4 the part takes the parent's `+0x91`.
/// TODO: a part with no draw state whose `+0x91 & 7` is 3 or 4 is then combined with its frame
/// by `0x5b1510` (modes 0, 1, 3, 4); not decoded, not done.
fn world(i: &mut Instance, pid: usize, local: Mat4, mode: u8) {
    let mut local = local;
    let mut world = local;
    let parent = i.parts[pid].parent.filter(|&q| q < i.parts.len());
    if let Some(q) = parent {
        let (pl, pw, b91) = (i.parts[q].local, i.parts[q].world, i.parts[q].b91);
        if let Some(pl) = pl {
            let moved = |m: &Mat4, by: &Mat4| *by * m.w_axis.xyz().extend(1.0);
            match mode {
                0 => {
                    world.w_axis = moved(&world, &pl);
                    local.w_axis = moved(&local, &pl);
                    i.parts[pid].b91 = b91;
                }
                3 => {
                    if let Some(pw) = pw {
                        world = pw * world;
                    }
                    local = pl * local;
                    i.parts[pid].b91 = b91;
                }
                4 => {
                    if let Some(pw) = pw {
                        world = mul3(world, normalised(pw));
                        world.w_axis = moved(&world, &pw);
                    }
                    local = mul3(local, normalised(pl));
                    local.w_axis = moved(&local, &pl);
                    i.parts[pid].b91 = b91;
                }
                _ => {}
            }
        }
    }
    let part = &mut i.parts[pid];
    if let Some(l) = part.local.as_mut() {
        *l = local;
    }
    if let Some(w) = part.world.as_mut() {
        *w = world;
    }
}

// --- generators ---

/// The spawn info a generator hands its child (`0x580405`, `0x581c43`): flags 7, position,
/// velocity, acceleration.
fn child_info(pos: Vec4, vel: Vec4, acc: Vec4) -> SpawnInfo {
    let mut v = [Vec4::ZERO; 12];
    v[0] = pos;
    v[1] = vel;
    v[2] = acc;
    SpawnInfo { flags: 7, v }
}

/// Where a child starts and which way it goes (`0x5b23b0`, with normalising mode 1): `off` the
/// random cube offset (w 1), `e` the emitter's translation (y, z negated), `frame` the
/// emitter's frame. Types 0, 2..8 and >= 10: position = direction = the offset (the emitter's
/// transform reaches the child later through FecLmatrix's parent step). Type 1: position =
/// offset moved by `e` (`0x58c550`, INFERRED a point transform), direction = the offset.
/// Type 9: the same through `e` then the frame (world space). The direction is normalised by
/// `0x58c6c0` (zero stays zero; w = 1).
fn place(ctype: u8, off: Vec4, e: Mat4, frame: Mat4) -> (Vec4, Vec4) {
    let (pos, dir) = match ctype {
        1 => {
            let p = e * off;
            (p, (p.xyz() - e.w_axis.xyz()).extend(1.0))
        }
        9 => {
            let m = frame * e;
            let p = m * off;
            (p, (p.xyz() - m.w_axis.xyz()).extend(1.0))
        }
        _ => (off, off),
    };
    let d = dir.xyz();
    let l2 = d.length_squared();
    let dir = if l2 == 0.0 { Vec3::ZERO } else { d / l2.sqrt() };
    (pos, dir.extend(1.0))
}

/// Count of a part group if a generator's child ref is one (`0x5b0e80`: bit 12), else -1.
fn group_count(i: &Instance, child: i16) -> i32 {
    if child < 0 || child & 0x1000 == 0 {
        return -1;
    }
    i.fep.bodies.get(i.body).and_then(|b| b.part_groups.get((child & 0xfff) as usize)).map_or(0, |g| g.len() as i32)
}

/// One child of a generator (`0x580485..0x580679`, `0x581cff..0x581ef6`): a random cube offset
/// (each axis in [-r, r), `+0x04`), placed by the child's matrix type, velocity = direction x
/// speed (`+0x08`), acceleration = direction x `+0x0c`, y and z of all three negated, spawned
/// at time 0 with this part as parent. Returns false when the spawn failed (the generator then
/// stops, `0x580681`, `0x581efe`).
fn spawn_child(i: &mut Instance, pid: usize, emitter_at: usize, p: usize) -> bool {
    let fep = i.fep.clone();
    let child = pi16(&fep, p + 0x10);
    let r = pf(&fep, p + 4);
    let off = Vec4::new(rng::signed(i.rng.as_mut(), r), rng::signed(i.rng.as_mut(), r), rng::signed(i.rng.as_mut(), r), 1.0);
    let em = i.parts[pid].rd4(emitter_at);
    let e = Mat4::from_cols(Vec4::X, Vec4::Y, Vec4::Z, Vec4::new(em.x, -em.y, -em.z, 1.0));
    spawn_at(i, pid, child, off, e, pf(&fep, p + 8), pf(&fep, p + 0x0c))
}

/// Steps 4..8 of a child (generators.md 2.2): placed by its matrix type from offset `off` and
/// matrix `e`, velocity / acceleration = direction x `speed` / `accel`, negated, spawned.
fn spawn_at(i: &mut Instance, pid: usize, child: i16, off: Vec4, e: Mat4, speed: f32, accel: f32) -> bool {
    let fep = i.fep.clone();
    let ctype = fep
        .bodies
        .get(i.body)
        .and_then(|b| b.parts.get((child as u16 & 0xfff) as usize))
        .and_then(|q| q.as_ref())
        .map_or(0, |q| q.matrix_type);
    let frame = match i.parts[pid].owner {
        Some(o) => i.parts[o].frame.unwrap_or(Mat4::IDENTITY),
        None => i.effect,
    };
    let (pos, dir) = place(ctype, off, e, frame);
    let acc = (dir.xyz() * accel).extend(0.0);
    let vel = (dir.xyz() * speed).extend(dir.w);
    let neg = |v: Vec4| Vec4::new(v.x, -v.y, -v.z, v.w);
    let info = child_info(neg(pos), neg(vel), neg(acc));
    i.spawn(Some(pid), child as i32, 0.0, Some(info)).is_some()
}

/// FecGenerateanchor's burst (`0x580350`): `+0x00` count (truncated), `+0x04` cube, `+0x08`
/// speed, `+0x0c` acceleration, `+0x10` child (< 0 off), `+0x12` (< 0 off). Input 0 = the
/// emitter (its output position).
/// Not done: a child ref with bit 12 (a part group, `0x5806a0..0x5807f9`): no file uses one,
/// and the groups' refs are file-wide numbers (README section 5).
fn anchor_fire(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    if pi16(&fep, p + 0x10) < 0 || pi16(&fep, p + 0x12) < 0 {
        return;
    }
    let Some(slot) = slot_of(i, pid, s) else { return };
    if group_count(i, pi16(&fep, p + 0x10)) >= 0 {
        return;
    }
    let mut k = 0;
    while k < pf(&fep, p) as i32 {
        if !spawn_child(i, pid, input_work(&slot, 0), p) {
            break;
        }
        k += 1;
    }
}

/// FecGenerate's slot 2 (`0x581a20`): continuous emission. `+0x00` rate per frame, `+0x12`
/// interval, `+0x17` chance %, the rest as FecGenerateanchor. Work: f32 accumulator.
fn generate(i: &mut Instance, pid: usize, s: usize, at: usize, p: usize, t: f32) {
    let fep = i.fep.clone();
    if pi16(&fep, p + 0x10) < 0 || pi16(&fep, p + 0x14) < 0 {
        return;
    }
    let rate = pf(&fep, p);
    let mut acc = i.parts[pid].rd(at);
    if rate == 0.0 && acc == 0.0 {
        return;
    }
    let dt = i.dt;
    if dt == 0.0 {
        return;
    }
    let interval = pu16(&fep, p + 0x12);
    let total = if 0.0 > dt {
        rate
    } else {
        // Near the end of a non-looping part's life: the frames left, counted in intervals
        // (`0x581b03..0x581b7f`).
        let mut steps = dt;
        let part = &i.parts[pid];
        if part.flags & 2 == 0 {
            let (pt, life) = (part.time, part.lifetime as f32);
            if pt + dt >= life {
                steps = life - pt;
                if dt > 1.0 {
                    steps = steps.max(1.0);
                }
                if steps > 1.0 {
                    let iv = if 0.0 < interval as f32 { interval as f32 } else { 1.0 };
                    let mut n = 0;
                    loop {
                        steps -= iv;
                        n += 1;
                        if !(steps > 1.0) {
                            break;
                        }
                    }
                    steps = n as f32;
                }
            }
        }
        // First frame of a one-a-frame emitter (`0x581b82..0x581bb9`).
        if 1.0 > t && acc == 0.0 && rate == 1.0 && 1.0 > dt && dt > 1.0 - (dt + t) {
            acc = 1.0 - dt;
            i.parts[pid].wr(at, acc);
        }
        let mut total = rate * steps + acc;
        let threshold = if interval > 0 { 1.0 } else { rate };
        if threshold > total {
            i.parts[pid].wr(at, total);
            return;
        }
        if 1.0 > dt {
            total = rate;
        }
        total
    };
    let n = total as i32;
    i.parts[pid].wr(at, total - n as f32 - interval as f32 * rate);
    let Some(slot) = slot_of(i, pid, s) else { return };
    if group_count(i, pi16(&fep, p + 0x10)) >= 0 {
        return; // part groups: see anchor_fire
    }
    let chance = pu8(&fep, p + 0x17);
    for _ in 0..n.max(0) {
        if chance < 100 && chance as i32 <= (rng::unit(i.rng.as_mut()) * 100.0) as i32 {
            continue;
        }
        if !spawn_child(i, pid, input_work(&slot, 0), p) {
            break;
        }
    }
}

// --- FecRnd* (22-25) ---

/// FecRndpos/rot/scl/col slot 1 (`0x5a9510`, `0x5a9ea0`, `0x5aa800`, `0x5a8a80`): skipped once
/// the part has looped (state bit 2). For k = value, velocity, acceleration: if its enable byte
/// is set, a random per component by its mode byte (`rng::by_mode`) is added to input 0's base
/// (and the output = the new base; FecRndscl x0.01), velocity or acceleration. Offset tables
/// (`0x9e82fc..0x9e8337`): enable / mode / range bytes.
fn rnd(i: &mut Instance, pid: usize, s: usize, id: u16, p: usize) {
    if i.parts[pid].state & LOOPED != 0 {
        return;
    }
    let fep = i.fep.clone();
    let Some(slot) = slot_of(i, pid, s) else { return };
    let w = input_work(&slot, 0);
    let (enable, mode): ([usize; 3], [usize; 3]) = match id {
        22 | 23 => ([0x0c, 0x0e, 0x1c], [0x0d, 0x0f, 0x1d]),
        24 => ([0x0d, 0x0f, 0x1d], [0x0e, 0x1c, 0x1e]),
        _ => ([0x30, 0x32, 0x34], [0x31, 0x33, 0x35]),
    };
    for k in 0..3 {
        if pu8(&fep, p + enable[k]) == 0 {
            continue;
        }
        let r = pv4(&fep, p + 0x10 * k);
        let m = pu8(&fep, p + mode[k]);
        let mut draw = |range: f32| rng::by_mode(rng::unit(i.rng.as_mut()), range, m);
        let d = match id {
            24 => {
                let x = draw(r.x);
                if pu8(&fep, p + 0x0c) == 0 {
                    Vec4::new(x, x, x, 0.0)
                } else {
                    let y = draw(r.y);
                    Vec4::new(x, y, draw(r.z), 0.0)
                }
            }
            25 => {
                let (x, y, z) = (draw(r.x), draw(r.y), draw(r.z));
                Vec4::new(x, y, z, draw(r.w))
            }
            _ => {
                let (x, y) = (draw(r.x), draw(r.y));
                Vec4::new(x, y, draw(r.z), 0.0)
            }
        };
        let part = &mut i.parts[pid];
        let at = w + [BASE, VEL, ACC][k];
        let v = part.rd4(at) + d;
        if id == 25 {
            part.wr4(at, v);
        } else {
            part.wr3(at, v.xyz());
        }
        if k == 0 {
            match id {
                24 => part.wr3(w + OUT, v.xyz() * PERCENT),
                25 => part.wr4(w + OUT, v),
                _ => part.wr3(w + OUT, v.xyz()),
            }
        }
    }
}

// --- drawing classes: their state ---

/// FecAnm_plus slot 0 (`0x586dc0`): frame 0, time = -dt plus a random start up to u16 `+0x18`,
/// flip by `+0x1a`: 1 -> bit 0, 2 -> bit 1 (half the time each), 3 -> 0..3.
/// Work: `+0` f32 time, `+4` u16 frame (0xffff ended), `+6` u8 flip.
fn anm_init(i: &mut Instance, pid: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let mut time = -i.dt;
    let rs = pu16(&fep, p + 0x18);
    if rs != 0 {
        time += rng::unit(i.rng.as_mut()) * rs as f32;
    }
    let flip = match pu8(&fep, p + 0x1a) {
        1 => (((rng::unit(i.rng.as_mut()) * 256.0) as i32) >= 0x80) as u8,
        2 => 2 * (((rng::unit(i.rng.as_mut()) * 256.0) as i32) >= 0x80) as u8,
        3 => ((rng::unit(i.rng.as_mut()) * 255.0) as i32 / 64) as u8,
        _ => 0,
    };
    let part = &mut i.parts[pid];
    part.wr_u16(at + 4, 0);
    part.wr(at, time);
    part.wr_u8(at + 6, flip);
}

/// FecAnm_plus slot 2 (`0x586c80`): unless ended, frame 0; with the ANM of section-4 index
/// `+0x0c` loaded: time += dt, t = time x speed (`+0x10`); past the ANM's length (`+0x14`): if
/// it loops (`+0x10 & 0x80`) t = int(t) mod length and time = t / speed, else ended; frame = the
/// key (12-byte keys at the ANM's `+0x2c`, duration at `+8`) whose summed durations pass t.
/// An ANM DDD would not have loaded (record id 0, or resident without resident.frr here) leaves
/// the clock where it is.
fn anm_update(i: &mut Instance, pid: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    if i.parts[pid].rd_u16(at + 4) == 0xffff {
        return;
    }
    i.parts[pid].wr_u16(at + 4, 0);
    let idx = pu32(&fep, p + 0x0c) as i32;
    let resident = i.resident.clone();
    let Some(anm) = usize::try_from(idx).ok().and_then(|k| fep.anm.get(k)).and_then(|r| res::data(&fep, resident.as_deref(), r)) else {
        return;
    };
    let rd = |o: usize| anm.get(o..o + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let part = &mut i.parts[pid];
    let mut time = part.rd(at) + i.dt;
    part.wr(at, time);
    let speed = pf(&fep, p + 0x10);
    let mut t = speed * time;
    let len = rd(0x14) as i32;
    if t >= len as f32 {
        if rd(0x10) & 0x80 == 0 || len == 0 {
            part.wr_u16(at + 4, 0xffff);
            return;
        }
        t = ((t as i32).wrapping_rem(len)) as f32;
        time = t / speed;
        part.wr(at, time);
    }
    let keys = rd(0x2c) as i32 as isize;
    let count = rd(0x1c) as usize;
    let dur = |k: usize| (keys + 8 + 12 * k as isize).try_into().ok().map(|o: usize| rd(o) as i32 as f32);
    let mut sum = dur(0).unwrap_or(0.0);
    let mut k = 0;
    while t >= sum && k + 1 < count.max(1) {
        k += 1;
        sum += dur(k).unwrap_or(0.0);
    }
    part.wr_u16(at + 4, k as u16);
}

/// FecMdlts_plus slot 0 (`0x588f60`): work `+0x00` = `+0x08` = value (param `+0x10`), `+0x10` =
/// velocity (`+0x18`), `+0x18` = acceleration (`+0x20`), three vec2.
fn mdlts_init(i: &mut Instance, pid: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let part = &mut i.parts[pid];
    let v = pv2(&fep, p + 0x10);
    part.wr2(at, v);
    part.wr2(at + 8, v);
    part.wr2(at + 0x10, pv2(&fep, p + 0x18));
    part.wr2(at + 0x18, pv2(&fep, p + 0x20));
}

/// FecMdlts_plus slot 1 (`0x588c80`): the ac rebase on the vec2 (output not refreshed), then,
/// unless the part has looped and if byte `+0x4a`, randoms in [-r, r) from `+0x28..+0x3c` added
/// to value, velocity, acceleration (x then y each).
fn mdlts_key(i: &mut Instance, pid: usize, at: usize, new: usize, old: Option<usize>, t: f32) {
    let Some(old) = old else { return };
    let fep = i.fep.clone();
    let part = &mut i.parts[pid];
    let (mut b, mut v, mut a) = (part.rd2(at + 8), part.rd2(at + 0x10), part.rd2(at + 0x18));
    let (np, nv, na) = (pv2(&fep, new + 0x10), pv2(&fep, new + 0x18), pv2(&fep, new + 0x20));
    let (op, ov, oa) = (pv2(&fep, old + 0x10), pv2(&fep, old + 0x18), pv2(&fep, old + 0x20));
    let tt = t * t;
    for k in 0..2 {
        let cur = a[k] * 0.5 * tt + v[k] * t + b[k];
        let was = oa[k] * 0.5 * tt + ov[k] * t + op[k];
        b[k] = cur - was + np[k];
        v[k] = (a[k] * t + v[k]) - (oa[k] * t + ov[k]) + nv[k];
        a[k] = a[k] - oa[k] + na[k];
    }
    if part.state & LOOPED == 0 && pu8(&fep, new + 0x4a) != 0 {
        let mut r = |o: usize| rng::signed(i.rng.as_mut(), pf(&fep, new + o));
        b.x += r(0x28);
        b.y += r(0x2c);
        v.x += r(0x30);
        v.y += r(0x34);
        a.x += r(0x38);
        a.y += r(0x3c);
    }
    let part = &mut i.parts[pid];
    part.wr2(at + 8, b);
    part.wr2(at + 0x10, v);
    part.wr2(at + 0x18, a);
}

/// FecMdlts_plus slot 5 (`0x589770`): the ac loop rewind on the vec2 (as `0x5b04e0`), then the
/// value wrapped into [0, 4096) (`[0x7b7ac4]` 4096).
fn mdlts_rewind(i: &mut Instance, pid: usize, at: usize, new: usize, t_new: f32, old: Option<usize>, t_old: f32) {
    let Some(old) = old else { return };
    let fep = i.fep.clone();
    let part = &mut i.parts[pid];
    let (mut b, mut v, mut a) = (part.rd2(at + 8), part.rd2(at + 0x10), part.rd2(at + 0x18));
    let (nv, na) = (pv2(&fep, new + 0x18), pv2(&fep, new + 0x20));
    let (ov, oa) = (pv2(&fep, old + 0x18), pv2(&fep, old + 0x20));
    for k in 0..2 {
        let v2 = (a[k] * t_old + v[k]) - (t_old * oa[k] + ov[k]) + nv[k];
        let a2 = a[k] - oa[k] + na[k];
        let cur = a[k] * 0.5 * (t_old * t_old) + v[k] * t_old + b[k];
        let mut nb = cur - (a2 * 0.5 * (t_new * t_new) + v2 * t_new);
        if 0.0 > nb || nb >= 4096.0 {
            nb += 4096.0;
            nb -= ((nb * (1.0 / 4096.0)) as i32) as f32 * 4096.0;
        }
        b[k] = nb;
        v[k] = v2;
        a[k] = a2;
    }
    part.wr2(at + 8, b);
    part.wr2(at + 0x10, v);
    part.wr2(at + 0x18, a);
}

// --- FecMegaanm_plus (153): a ring buffer of sprites (docs/fep/draw-sprites.md 5) ---

/// One sprite: the 0x80-byte record (5.2). Everything is its value at spawn; the draw evaluates
/// `x0 + v age + a age^2 / 2` (nothing is integrated per frame).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Rec {
    /// `+0x00` colour RGBA, `+0x10` its velocity.
    col: Vec4,
    col_vel: Vec4,
    /// `+0x20` position, `+0x2c` velocity, `+0x38` acceleration (part-local or world).
    pos: Vec3,
    vel: Vec3,
    acc: Vec3,
    /// `+0x44` scale percent (z = 100), `+0x50` velocity, `+0x5c` acceleration.
    scale: Vec3,
    scale_vel: Vec3,
    scale_acc: Vec3,
    /// `+0x68` age in frames.
    age: f32,
    /// `+0x6c` screen angle (degrees), spin, spin acceleration.
    rot: Vec3,
    /// `+0x78` ANM time, `+0x7c` ANM key, `+0x7e` flip bits.
    anm_t: f32,
    frame: u16,
    flip: u8,
}

/// The buffer (work `+0`; DDD allocates `0x30 + 0x80 N` bytes, `0x5bd400`, not zeroed: the
/// header fields only the key writes start at 0 here).
#[derive(Debug, Clone)]
pub(crate) struct Mega {
    /// `+0x04` emission end (0 none, -1 forever), `+0x08` life, `+0x0c` fade-out start age,
    /// `+0x10` fade-in rate, `+0x14` fade-out rate, `+0x18` time since the last key.
    emit_end: f32,
    life: f32,
    fade_start: f32,
    fade_in: f32,
    fade_out: f32,
    t_key: f32,
    /// `+0x1c` N, `+0x1e` live count, `+0x20` head (the oldest), `+0x22` velocity-to-world.
    n: u16,
    count: u16,
    head: u16,
    world_vel: bool,
    recs: Vec<Rec>,
}

// Work: `+0x08` f32 emission accumulator, `+0x0c` f32 last amount, `+0x10` s16 vertex cursor.
const MEGA_ACC: usize = 0x08;
const MEGA_LAST: usize = 0x0c;
const MEGA_CURSOR: usize = 0x10; // shapes 8, 9 only (not implemented)

fn pv3(f: &Fep, at: usize) -> Vec3 {
    pv4(f, at).truncate()
}

/// `0x58c6c0`: v / |v|, zero stays zero.
fn norm(v: Vec3) -> Vec3 {
    let l2 = v.length_squared();
    if l2 == 0.0 {
        Vec3::ZERO
    } else {
        v / l2.sqrt()
    }
}

/// `RotXYZ(a)` (`0x58c660`): identity, then `RotZ`, `RotY`, `RotX` (each `in * R`): DDD
/// `Rz * Ry * Rx`, glam `Rx * Ry * Rz`.
fn rot_xyz(a: Vec3) -> Mat4 {
    rot(Axis::X, a.x) * rot(Axis::Y, a.y) * rot(Axis::Z, a.z)
}

/// `LookAt(0, d)` (`0x582b20`): up = z when d.x = d.z = 0, else y; X = norm(up x d), Z =
/// norm(d), Y = Z x X; DDD rows X, Y, Z, (0, 0, 0, 1) = glam columns.
fn look_at(d: Vec3) -> Mat4 {
    let up = if d.x == 0.0 && d.z == 0.0 { Vec3::Z } else { Vec3::Y };
    let x = norm(up.cross(d));
    let z = norm(d);
    let y = z.cross(x);
    Mat4::from_cols(x.extend(0.0), y.extend(0.0), z.extend(0.0), Vec4::W)
}

/// The cone frame `B = RotXYZ(a) * LookAt(0, axis)` (DDD order; glam `LookAt * RotXYZ`).
fn cone(a: Vec3, axis: Vec3) -> Mat4 {
    look_at(axis) * rot_xyz(a)
}

/// `frame()` (`0x58c510`): the frame owner's `+0x88`, else the effect matrix `+0x60`.
fn frame_of(i: &Instance, pid: usize) -> Mat4 {
    match i.parts[pid].owner {
        Some(o) => i.parts[o].frame.unwrap_or(Mat4::IDENTITY),
        None => i.effect,
    }
}

/// The ANM of a block (section-4 index `+0x0c`), as the slots test it: index >= 0, a record,
/// bytes loaded (resident ones through resident.frr).
fn anm_of<'a>(fep: &'a Fep, resident: Option<&'a res::Resident>, p: usize) -> Option<&'a [u8]> {
    let idx = pu32(fep, p + 0x0c) as i32;
    usize::try_from(idx).ok().and_then(|k| fep.anm.get(k)).and_then(|r| res::data(fep, resident, r))
}

/// Slot 0 (`0x58df30`): the buffer for N (`+0xf0`, the init block's) sprites; count = head =
/// 0; the accumulators and cursor 0; velocity-to-world when space `+0xfb` is 7 and the matrix
/// type is 0, 3 or 4 (`0x58df96..0x58dfc3`).
fn mega_init(i: &mut Instance, pid: usize, s: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let n = pu16(&fep, p + 0xf0);
    let world_vel = pu8(&fep, p + 0xfb) == 7 && matches!(i.parts[pid].matrix_type, 0 | 3 | 4);
    let part = &mut i.parts[pid];
    if let Some(m) = part.mega.get_mut(s) {
        *m = Some(Box::new(Mega {
            emit_end: 0.0,
            life: 0.0,
            fade_start: 0.0,
            fade_in: 0.0,
            fade_out: 0.0,
            t_key: 0.0,
            n,
            count: 0,
            head: 0,
            world_vel,
            recs: vec![Rec::default(); n as usize],
        }));
    }
    part.wr(at + MEGA_ACC, 0.0);
    part.wr(at + MEGA_LAST, 0.0);
    part.wr_u16(at + MEGA_CURSOR, 0);
}

/// Slot 1 (`0x58cc40`), nothing once the part has looped: emission end from the rate `+0x2c`
/// and its "acceleration" `+0x3c`, the key clock 0, life `+0xf2`, the fades `+0x110..0x112`.
fn mega_key(i: &mut Instance, pid: usize, s: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let part = &mut i.parts[pid];
    if part.state & LOOPED != 0 {
        return;
    }
    let Some(Some(m)) = part.mega.get_mut(s) else { return };
    let (rate, racc) = (pf(&fep, p + 0x2c), pf(&fep, p + 0x3c));
    m.emit_end = if rate <= 0.0 {
        0.0
    } else if racc >= 0.0 {
        -1.0
    } else {
        -(rate / racc)
    };
    m.t_key = 0.0;
    let life = pu16(&fep, p + 0xf2) as i32;
    m.life = life as f32;
    if pu8(&fep, p + 0x110) == 0 {
        m.fade_in = 1.0;
        m.fade_out = 0.0;
        m.fade_start = 0.0;
    } else {
        let (fi, fo) = (pu8(&fep, p + 0x111) as i32, pu8(&fep, p + 0x112) as i32);
        let out = if fo != 0 { 1.0 / fo as f32 } else { 0.0 };
        if fi + fo < life {
            m.fade_start = (life - fo) as f32;
            m.fade_out = out;
            m.fade_in = if fi != 0 { 1.0 / fi as f32 } else { 1.0 };
        } else if fo < life {
            m.fade_start = (life - fo) as f32;
            m.fade_in = 1.0 / (life - fo) as f32;
            m.fade_out = out;
        } else {
            m.fade_start = 0.0;
            m.fade_in = 0.0;
            m.fade_out = 1.0 / fo as f32;
        }
    }
    part.wr(at + MEGA_LAST, rate);
}

/// What one spawn needs that does not change within an update.
struct SpawnCtx {
    mt: u8,
    local: Mat4,
    frame: Mat4,
}

/// Slot 2 (`0x58cdd0`): emission (`0x58ce92..0x58dcb0`), then ageing (`0x58dcfe..0x58de93`),
/// with the element's dt; nothing at dt 0.
fn mega_update(i: &mut Instance, pid: usize, s: usize, at: usize, p: usize) {
    let dt = i.dt;
    let Some(mut m) = i.parts[pid].mega.get_mut(s).and_then(Option::take) else { return };
    if dt != 0.0 {
        mega_emit(i, pid, at, p, &mut m, dt);
        mega_age(i, p, &mut m, dt);
    }
    i.parts[pid].mega[s] = Some(m);
}

fn mega_emit(i: &mut Instance, pid: usize, at: usize, p: usize, m: &mut Mega, dt: f32) {
    let fep = i.fep.clone();
    m.t_key += dt;
    if m.emit_end == 0.0 || (m.emit_end > 0.0 && m.t_key > m.emit_end) {
        return;
    }
    let (rate, racc) = (pf(&fep, p + 0x2c), pf(&fep, p + 0x3c));
    let amount = dt * dt * (racc * 0.5) + dt * rate;
    i.parts[pid].wr(at + MEGA_LAST, amount);
    if amount <= 0.0 {
        return;
    }
    let sum = i.parts[pid].rd(at + MEGA_ACC) + amount;
    if sum < 1.0 {
        i.parts[pid].wr(at + MEGA_ACC, sum);
        return;
    }
    let n = sum as i32;
    let step = dt / sum;
    let iv = pu8(&fep, p + 0xfc) as f32;
    i.parts[pid].wr(at + MEGA_ACC, (sum - n as f32) - (iv * amount + iv * iv * (racc * 0.5)));
    let cx = SpawnCtx { mt: i.parts[pid].matrix_type, local: i.parts[pid].local.unwrap_or(Mat4::IDENTITY), frame: frame_of(i, pid) };
    let chance = pu8(&fep, p + 0xfd);
    for r in (1..=n).rev() {
        if chance < 100 && chance as i32 <= (rng::unit(i.rng.as_mut()) * 100.0) as i32 {
            continue;
        }
        if m.count >= m.n {
            break; // full: the rest of this frame's sprites are lost
        }
        let k = (m.head as usize + m.count as usize) % m.n as usize;
        m.count += 1;
        let world_vel = m.world_vel;
        mega_spawn(&fep, i.rng.as_mut(), p, &cx, world_vel, &mut m.recs[k], r as f32 * step - 1.0);
    }
}

/// One sprite (`0x58d0a8..0x58dc8b`): age and ANM time = `r step - 1`, flip, position by shape,
/// velocity, scale, rotation, colour, drawing randoms in DDD's order (5.5).
fn mega_spawn(fep: &Fep, rng: &mut dyn Rng, p: usize, cx: &SpawnCtx, world_vel: bool, rec: &mut Rec, age: f32) {
    let f = |o: usize| pf(fep, p + o);
    let b = |o: usize| pu8(fep, p + o);
    rec.frame = 0;
    rec.age = age;
    rec.anm_t = age;
    // irand(n) = int(rand01 n) (`0x5811a0`).
    fn irand(rng: &mut dyn Rng, n: f32) -> i32 {
        (rng::unit(rng) * n) as i32
    }
    rec.flip = match b(0xfa) {
        1 => (irand(rng, 256.0) >= 128) as u8,
        2 => 2 * (irand(rng, 256.0) >= 128) as u8,
        3 => (irand(rng, 255.0) >> 6) as u8,
        _ => 0,
    };

    // Position (`0x58d119`, table `0x58ded8`).
    let shape = b(0xfe);
    let cone_deg = f(0xb0);
    let space = b(0xfb);
    let place = |pt: Vec3| -> (Vec3, Vec3) {
        let w = cx.frame * cx.local; // DDD L * frame()
        if space == 7 && matches!(cx.mt, 0 | 3 | 4) {
            let pos = w.transform_point3(pt);
            let sc = Vec3::new(w.x_axis.truncate().length(), w.y_axis.truncate().length(), w.z_axis.truncate().length());
            (pos, norm(pos - w.w_axis.truncate()) * sc)
        } else if cx.mt == 2 || (6..=9).contains(&cx.mt) {
            let pos = w.transform_point3(pt);
            (pos, norm(pos - w.w_axis.truncate()))
        } else {
            (pt, norm(pt))
        }
    };
    let mut d = Vec3::ZERO;
    match shape {
        0 | 2 => {
            let h = pv3(fep, p);
            let x = rng::signed(rng, h.x);
            let y = rng::signed(rng, h.y);
            let pt = Vec3::new(x, y, rng::signed(rng, h.z));
            (rec.pos, d) = place(pt);
        }
        1 | 3 | 6 | 7 => {
            let a = if shape <= 3 {
                let x = rng::signed(rng, cone_deg);
                let y = rng::signed(rng, cone_deg);
                Vec3::new(x, y, rng::signed(rng, cone_deg))
            } else {
                Vec3::new(0.0, rng::signed(rng, cone_deg), 0.0)
            };
            let mut ax = pv3(fep, p + 0x10);
            if ax.x == 0.0 && ax.y == 0.0 {
                ax.x = 1e-6;
            }
            ax.y = -ax.y;
            ax.z = -ax.z;
            let bm = cone(a, ax);
            let big_r = f(0);
            let pc = b(0x102) as f32 / 100.0;
            let radius = if b(0x100) == 0 {
                rng::signed(rng, big_r * 0.5) + big_r * 0.5
            } else {
                match b(0x101) {
                    0 => big_r - rng::unit(rng) * pc * big_r,
                    1 => rng::unit(rng) * pc * big_r,
                    _ => cone_deg, // a stale register (`0x58d2b7`); not in the data
                }
            };
            (rec.pos, d) = place(bm.z_axis.truncate() * radius);
        }
        // Shapes 8, 9 (a VTL vertex of an MDL, as Generateanchorvertex): not implemented here;
        // taken as DDD's "missing piece" case: position 0, no direction (`0x58d694`).
        8 | 9 => rec.pos = Vec3::ZERO,
        _ => {} // 4, 5, > 9: the record keeps what it had
    }

    // Velocity (`0x58d656..`, table `0x58df00`).
    rec.vel = Vec3::ZERO;
    rec.acc = Vec3::ZERO;
    let (mut spd, mut acc) = if b(0x103) != 0 { (f(0xb4), f(0xb8)) } else { (0.0, 0.0) };
    if b(0x104) != 0 {
        spd += rng::signed(rng, f(0xbc));
        acc += rng::signed(rng, f(0xc0));
    }
    match shape {
        0 | 1 | 6 | 8 => {
            let x = rng::signed(rng, cone_deg);
            let y = -rng::signed(rng, cone_deg);
            let a = Vec3::new(x, y, -rng::signed(rng, cone_deg));
            let mut ax = pv3(fep, p + 0x10);
            ax.y = -ax.y;
            ax.z = -ax.z;
            if ax.x == 0.0 && ax.y == 0.0 {
                ax.x = 1e-6;
            }
            let mut bm = Mat3::from_mat4(cone(a, ax));
            if world_vel {
                // `0x582810` twice: DDD B * L, then * frame().
                bm = Mat3::from_mat4(cx.frame) * (Mat3::from_mat4(cx.local) * bm);
            }
            let u = bm.x_axis * 1e-5 + bm.z_axis;
            rec.vel = u * spd;
            rec.acc = u * acc;
        }
        2 | 3 | 7 | 9 => {
            rec.vel = d * spd;
            rec.acc = d * acc;
        }
        _ => {}
    }
    let mut e = pv3(fep, p + 0x20);
    let (mut spd2, mut acc2) = if b(0x105) != 0 { (f(0xc4), f(0xc8)) } else { (0.0, 0.0) };
    if b(0x106) != 0 {
        e.x += rng::signed(rng, f(0x30));
        e.y += rng::signed(rng, f(0x34));
        e.z += rng::signed(rng, f(0x38));
        spd2 += rng::signed(rng, f(0xcc));
        acc2 += rng::signed(rng, f(0xd0));
    }
    let e = norm(e);
    let neg = |v: Vec3| Vec3::new(v.x, -v.y, -v.z);
    rec.vel += neg(e * spd2);
    rec.acc += neg(e * acc2);

    // Scale (`0x58dab5..`, randoms `0x58e800`).
    rec.scale = Vec3::splat(100.0);
    rec.scale_vel = Vec3::ZERO;
    rec.scale_acc = Vec3::ZERO;
    if b(0x107) != 0 {
        rec.scale.x = f(0x80);
        rec.scale.y = f(0x84);
        rec.scale_vel.x = f(0x88);
        rec.scale_vel.y = f(0x8c);
        rec.scale_acc.x = f(0x90);
        rec.scale_acc.y = f(0x94);
    }
    if b(0x108) != 0 {
        match b(0x109) {
            0 => {
                let rb = rng::signed(rng, f(0x98));
                let rv = rng::signed(rng, f(0xa0));
                let ra = rng::signed(rng, f(0xa8));
                for c in 0..2 {
                    rec.scale[c] += rb;
                    rec.scale_vel[c] += rv;
                    rec.scale_acc[c] += ra;
                }
            }
            1 => {
                for c in 0..2 {
                    let (sr, vr, ar) = (f(0x98 + 4 * c), f(0xa0 + 4 * c), f(0xa8 + 4 * c));
                    rec.scale[c] += rec.scale[c] * 0.01 * rng::signed(rng, sr);
                    rec.scale_vel[c] += rec.scale_vel[c] * 0.01 * rng::signed(rng, vr);
                    rec.scale_acc[c] += rec.scale_acc[c] * 0.01 * rng::signed(rng, ar);
                    rec.scale[c] += rng::signed(rng, sr);
                    rec.scale_vel[c] += rng::signed(rng, vr);
                    rec.scale_acc[c] += rng::signed(rng, ar);
                }
            }
            _ => {}
        }
    }

    // Rotation, colour.
    rec.rot = if b(0x10a) != 0 { pv3(fep, p + 0xd4) } else { Vec3::ZERO };
    if b(0x10b) != 0 {
        for k in 0..3 {
            rec.rot[k] += rng::signed(rng, f(0xe0 + 4 * k));
        }
    }
    if b(0x10c) != 0 {
        rec.col = pv4(fep, p + 0x40);
        rec.col_vel = pv4(fep, p + 0x50);
    } else {
        rec.col = Vec4::splat(128.0);
        rec.col_vel = Vec4::ZERO;
    }
    if b(0x10d) != 0 {
        for c in 0..4 {
            rec.col[c] += rng::signed(rng, f(0x60 + 4 * c));
            rec.col_vel[c] += rng::signed(rng, f(0x70 + 4 * c));
        }
    }
}

/// Ageing (`0x58dcfe..0x58de93`), also on frames without emission; nothing without the ANM
/// (the sprites then never age or die). Per sprite, oldest first: expired (age >= life, or a
/// non-looping ANM past its end) counts as dead; else the ANM key from `anm_t x speed` (a
/// looping one wraps and keeps the **scaled** value), then age and ANM time += dt. The `dead`
/// oldest are dropped.
fn mega_age(i: &Instance, p: usize, m: &mut Mega, dt: f32) {
    let fep = &i.fep;
    let Some(anm) = anm_of(fep, i.resident.as_deref(), p) else { return };
    let rd = |o: usize| anm.get(o..o + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let speed = pf(fep, p + 0x1c);
    let looping = rd(0x10) & 0x80 != 0;
    let len = rd(0x14) as f32;
    let keys = rd(0x2c) as usize;
    let key_count = rd(0x1c) as usize;
    let dur = |k: usize| rd(keys + 8 + 12 * k) as i32 as f32;
    let mut dead = 0u16;
    for j in 0..m.count as usize {
        let rec = &mut m.recs[(m.head as usize + j) % m.n as usize];
        if rec.age >= m.life {
            dead += 1;
            continue;
        }
        let mut t = rec.anm_t;
        let mut x = t * speed;
        if x >= len {
            // A looping ANM of length 0 would divide by zero in DDD: taken as ended.
            if !looping || len as i32 == 0 {
                dead += 1;
                continue;
            }
            let xi = x as i32;
            x -= (xi - xi % len as i32) as f32;
            t = x;
        }
        let mut k = 0;
        let mut end = dur(0);
        while x >= end && k + 1 < key_count {
            k += 1;
            end += dur(k);
        }
        rec.frame = k.min(key_count.saturating_sub(1)) as u16;
        rec.age += dt;
        rec.anm_t = t + dt;
    }
    if m.n > 0 {
        m.head = ((m.head as usize + dead as usize) % m.n as usize) as u16;
    }
    m.count = m.count.saturating_sub(dead);
}

/// The live sprites of a Megaanm_plus slot as the draw (`0x58e010`, 5.4) evaluates them, oldest
/// first, skipping age >= life; none without the buffer or the ANM.
pub(crate) fn mega_sprites(i: &Instance, pid: usize, s: usize, p: usize) -> Vec<crate::run::Sprite> {
    let fep = &i.fep;
    let Some(Some(m)) = i.parts[pid].mega.get(s) else { return Vec::new() };
    if anm_of(fep, i.resident.as_deref(), p).is_none() || m.n == 0 {
        return Vec::new();
    }
    let world = matches!(pu8(fep, p + 0xfb), 2 | 6 | 7);
    let mut out = Vec::with_capacity(m.count as usize);
    for j in 0..m.count as usize {
        let rec = &m.recs[(m.head as usize + j) % m.n as usize];
        let a = rec.age;
        if a >= m.life {
            continue;
        }
        let mut c = rec.col + rec.col_vel * a;
        let f = if m.fade_start > a { a * m.fade_in } else { 1.0 - (a - m.fade_start) * m.fade_out };
        if 0.0 > f {
            c.w = 0.0;
        } else if 1.0 > f {
            c.w *= f;
        }
        let h = a * a * 0.5;
        out.push(crate::run::Sprite {
            anm_frame: rec.frame,
            flip: rec.flip,
            position: rec.pos + rec.vel * a + rec.acc * h,
            world,
            scale: rec.scale + rec.scale_vel * a + rec.scale_acc * h,
            angle: rec.rot.x + rec.rot.y * a + rec.rot.z * h,
            colour: c,
            age: a,
        });
    }
    out
}

// --- FecGenerateanchorvertex (150): children on model vertices (generators.md 5) ---

/// Work: `+0` s16 in-order cursor, `+2` u8 pending (`0x57fb60`).
const VTX_CURSOR: usize = 0;
const VTX_PENDING: usize = 2;

/// `[0x7ecfa0]` 2^-15: a mesh position's s16 to model units.
const INV_2_15: f32 = 1.0 / 32768.0;

/// The burst (`0x57fb70`): `+0x00` count, `+0x04` speed, `+0x08` acceleration, `+0x0c` child,
/// `+0x0e` MDL (section 3), `+0x10` VTL (section 5), `+0x12` list in it, `+0x14` (< 0 off),
/// `+0x16` bit 0 in order. P = the list's vertex of the MDL's first mesh (`s16 x 2^-15 x record
/// f32 x mesh f32 +0x24`, centred by the record's `+0x14` bit 0), M = the part's local matrix.
/// A model or list DDD would not have loaded (no bytes here) ends the burst before any draw.
fn vertex_fire(i: &mut Instance, pid: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let (child, mdl_i, vtl_i) = (pi16(&fep, p + 0x0c), pi16(&fep, p + 0x0e), pi16(&fep, p + 0x10));
    if child < 0 || pi16(&fep, p + 0x14) < 0 || mdl_i < 0 || vtl_i < 0 {
        return;
    }
    let resident = i.resident.clone();
    let Some(rec) = fep.mdl.get(mdl_i as usize) else { return };
    let Some(mdl) = res::data(&fep, resident.as_deref(), rec) else { return };
    let Some(vtl) = fep.vtl.get(vtl_i as usize).and_then(|r| res::data(&fep, resident.as_deref(), r)) else { return };
    let u32_of = |d: &[u8], o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let s16_of = |d: &[u8], o: usize| d.get(o..o + 2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32);
    // The record's `+0x14` (section 3 record `mdl_i`, 24 bytes each).
    let centre = pu32(&fep, fep.header.sections[3] as usize + 24 * mdl_i as usize + 0x14) & 1 != 0;
    // VTL `+0x18` -> list table {u32 count, u32 offset} (`0x5c0a60`).
    let li = pi16(&fep, p + 0x12) as i32 as usize;
    let Some(table) = u32_of(vtl, 0x18) else { return };
    let (Some(n), Some(off)) = (u32_of(vtl, table as usize + 8 * li), u32_of(vtl, table as usize + 8 * li + 4)) else { return };
    if off == 0 || n == 0 {
        return; // DDD: null list; n = 0 never in the data (it would read past the list)
    }
    let Some(mesh) = u32_of(mdl, 0x1c).map(|o| o as usize) else { return };
    let Some(pos) = u32_of(mdl, mesh + 4).map(|o| o as usize) else { return };
    let k = rec.scale * f32::from_bits(u32_of(mdl, mesh + 0x24).unwrap_or(0));
    let mean = Vec3::new(
        f32::from_bits(u32_of(mdl, mesh + 0x38).unwrap_or(0)),
        f32::from_bits(u32_of(mdl, mesh + 0x3c).unwrap_or(0)),
        f32::from_bits(u32_of(mdl, mesh + 0x40).unwrap_or(0)),
    );
    let m = i.parts[pid].local.unwrap_or(Mat4::IDENTITY);
    let in_order = pu8(&fep, p + 0x16) & 1 != 0;
    let mut c = 0;
    while c < pf(&fep, p) as i32 {
        let j = if in_order {
            let cur = i.parts[pid].rd_u16(at + VTX_CURSOR) as i16 as i32;
            let next = if cur + 1 >= n as i32 { 0 } else { cur + 1 };
            i.parts[pid].wr_u16(at + VTX_CURSOR, next as u16);
            cur.max(0) as usize
        } else {
            (rng::unit(i.rng.as_mut()) * n as f32) as i32 as usize
        };
        let Some(v) = s16_of(vtl, off as usize + 2 * j) else { break };
        let v = v as i32 as usize;
        let a = |c: usize| s16_of(mdl, pos + 6 * v + 2 * c).unwrap_or(0.0) * INV_2_15;
        let raw = Vec3::new(a(0), a(1), a(2));
        let pt = if centre { (raw - mean) * k } else { raw * k };
        if !spawn_at(i, pid, child, pt.extend(1.0), m, pf(&fep, p + 4), pf(&fep, p + 8)) {
            break;
        }
        c += 1;
    }
}

// --- FecColorblink_parameter (61), FecColorblink (62) (generators.md 7) ---

// Colorblink_parameter work: output factor, step, counter, phase ends, phase.
const BLINK_STEP: usize = 0x10;
const BLINK_COUNT: usize = 0x20;
const BLINK_ENDS: usize = 0x24;
const BLINK_PHASE: usize = 0x34;

fn wr_u32(i: &mut Instance, pid: usize, at: usize, v: u32) {
    i.parts[pid].wr(at, f32::from_bits(v));
}
fn rd_u32(i: &Instance, pid: usize, at: usize) -> u32 {
    i.parts[pid].rd(at).to_bits()
}

/// Slots 0 (`0x5852c0`) and 1 (`0x584ff0`): counter 0, phase ends `holdA`, `+ fade`, `+ holdB`,
/// `+ fade` from `+0x40..+0x44`, output (1, 1, 1, 1), step 0, phase 0.
fn blink_reset(i: &mut Instance, pid: usize, at: usize, p: usize) {
    let fep = i.fep.clone();
    let (ha, hb, fade) = (pu16(&fep, p + 0x40) as u32, pu16(&fep, p + 0x42) as u32, pu16(&fep, p + 0x44) as u32);
    let e0 = ha;
    let e1 = e0 + fade;
    let e2 = e1 + hb;
    for (k, e) in [e0, e1, e2, e2 + fade].into_iter().enumerate() {
        wr_u32(i, pid, at + BLINK_ENDS + 4 * k, e);
    }
    wr_u32(i, pid, at + BLINK_COUNT, 0);
    wr_u32(i, pid, at + BLINK_PHASE, 0);
    i.parts[pid].wr4(at + OUT, Vec4::ONE);
    i.parts[pid].wr4(at + BLINK_STEP, Vec4::ZERO);
}

/// `0x584cd0` (via `0x5851e0` B, `0x585250` A): the step toward colour X (`+0x00` A / `+0x20` B)
/// with its random range (`+0x10` / `+0x30`; on `+0x46` / `+0x48`, kind `+0x47` / `+0x49`: 2
/// negates the draw), clamped to [0, 1], over `fade` updates.
fn blink_target(i: &mut Instance, pid: usize, at: usize, p: usize, b: bool) {
    let fep = i.fep.clone();
    let (col, range, on, kind) = if b { (0x20, 0x30, 0x48, 0x49) } else { (0x00, 0x10, 0x46, 0x47) };
    let mut r = Vec4::ZERO;
    if pu8(&fep, p + on) != 0 {
        let rg = pv4(&fep, p + range);
        for c in 0..4 {
            let d = rng::signed(i.rng.as_mut(), rg[c]);
            r[c] = if pu8(&fep, p + kind) == 2 { -d } else { d };
        }
    }
    let t = (pv4(&fep, p + col) + r).clamp(Vec4::ZERO, Vec4::ONE);
    let out = i.parts[pid].rd4(at + OUT);
    let fade = pu16(&fep, p + 0x44);
    let step = if fade > 0 { (t - out) / fade as f32 } else { t - out };
    i.parts[pid].wr4(at + BLINK_STEP, step);
}

/// Slot 2 (`0x585050`): phase by the counter (hold A, fade to B, hold B, fade to A); entering a
/// phase sets the step (with fade 0, the jumps of `0x585150..`); output += step; counter + 1,
/// wrapping at the period. Once per update whatever dt is; nothing at dt 0.
fn blink_update(i: &mut Instance, pid: usize, at: usize, p: usize) {
    if i.dt == 0.0 {
        return;
    }
    let n = rd_u32(i, pid, at + BLINK_COUNT);
    let e = |k: usize| rd_u32(i, pid, at + BLINK_ENDS + 4 * k);
    let ph = if n < e(0) {
        0
    } else if n < e(1) {
        1
    } else if n < e(2) {
        2
    } else {
        3
    };
    let stored = rd_u32(i, pid, at + BLINK_PHASE);
    if ph != stored {
        let jump = |i: &mut Instance| {
            let part = &mut i.parts[pid];
            let o = part.rd4(at + OUT) + part.rd4(at + BLINK_STEP);
            part.wr4(at + OUT, o);
            part.wr4(at + BLINK_STEP, Vec4::ZERO);
        };
        match (ph, stored) {
            (1, _) => blink_target(i, pid, at, p, true),
            (3, _) => blink_target(i, pid, at, p, false),
            (2, 0) => {
                blink_target(i, pid, at, p, false);
                jump(i);
            }
            (0, 2) => {
                blink_target(i, pid, at, p, true);
                jump(i);
            }
            _ => i.parts[pid].wr4(at + BLINK_STEP, Vec4::ZERO),
        }
        wr_u32(i, pid, at + BLINK_PHASE, ph);
    }
    let part = &mut i.parts[pid];
    let o = part.rd4(at + OUT) + part.rd4(at + BLINK_STEP);
    part.wr4(at + OUT, o);
    let n = n + 1;
    let period = rd_u32(i, pid, at + BLINK_ENDS + 12);
    wr_u32(i, pid, at + BLINK_COUNT, if n >= period { 0 } else { n });
}

/// FecColorblink slot 2 (`0x584b40`): input 0's output (a Col*) = clamp(it x input 1's output,
/// 0, 255) per lane (`0x63b0c0`), in place.
fn colorblink(i: &mut Instance, pid: usize, s: usize) {
    let Some(slot) = slot_of(i, pid, s) else { return };
    let (c, f) = (input_work(&slot, 0), input_work(&slot, 1));
    let part = &mut i.parts[pid];
    let v = clamp255(part.rd4(c) * part.rd4(f));
    part.wr4(c, v);
}

// --- the trail: FecNull_routegenerator (79), FecLocus_tail_strategy (68), FecShapetail (80)
// (tails.md 1) ---

/// The LocusStrategy object (`0x5a5fc0`, 0xd8 bytes; tails.md 1.3), kept beside the part (DDD:
/// its pointer at work `+0`). Matrices in glam's form.
#[derive(Debug, Clone)]
pub(crate) struct Locus {
    /// `+0x08` M (the sample space), `+0x48` W.
    m: Mat4,
    w: Mat4,
    /// `+0x90` the points (P[0] newest), `+0x98` spacing per gap, `+0xa0` / `+0xa8` edges.
    pub(crate) p: Vec<Vec4>,
    spacing: Vec<f32>,
    edge_a: Vec<Vec4>,
    edge_b: Vec<Vec4>,
    /// `+0xb0` points reached (1..N), `+0xb8` progress to the next.
    pub(crate) v: usize,
    progress: f32,
    /// `+0xc0` history ring, `+0xcc` head (newest), `+0xd0` entries filled.
    hist: Vec<Vec4>,
    head: usize,
    filled: usize,
    /// `+0xd4` space (world), `+0xd5` first update pending.
    pub(crate) space: bool,
    first: bool,
}

/// `0x63d030`: the inverse, identity when singular.
fn inverse(m: Mat4) -> Mat4 {
    if m.determinant() == 0.0 {
        Mat4::IDENTITY
    } else {
        m.inverse()
    }
}

/// vt[3] (`0x5a6180`): space 0 (`0x5a55a0`) M = the part's local, W = its world (or M); space 1
/// both times the frame (DDD `local x F`).
fn locus_mats(i: &Instance, pid: usize, space: bool) -> (Mat4, Mat4) {
    let part = &i.parts[pid];
    let l = part.local.unwrap_or(Mat4::IDENTITY);
    let w = part.world.unwrap_or(l);
    if space {
        let f = frame_of(i, pid);
        (f * l, f * w)
    } else {
        (l, w)
    }
}

/// Slot 0 (`0x5a2110`): construct, `init` (`0x5a6b80`) with N = s16 `+0x20` + 1 and history
/// request u16 `+0x22`, then `reset`.
fn locus_init(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let n = (pi16(&fep, p + 0x20) as i32 + 1).max(1) as usize;
    let space = pu8(&fep, p + 0x26) != 0;
    let hc = (pu16(&fep, p + 0x22) as usize).max(if space { 3 } else { 2 } * n);
    let l = Locus {
        m: Mat4::IDENTITY,
        w: Mat4::IDENTITY,
        p: vec![Vec4::ZERO; n],
        spacing: vec![0.0; n],
        edge_a: vec![Vec4::ZERO; n],
        edge_b: vec![Vec4::ZERO; n],
        v: 0,
        progress: 0.0,
        hist: vec![Vec4::ZERO; hc],
        head: 0,
        filled: 0,
        space,
        first: true,
    };
    if let Some(slot) = i.parts[pid].locus.get_mut(s) {
        *slot = Some(Box::new(l));
    }
    locus_reset(i, pid, s, p);
}

/// The route point: input 1's output (FecNull_routegenerator: 0).
fn route(i: &Instance, pid: usize, s: usize) -> Vec4 {
    let Some(slot) = slot_of(i, pid, s) else { return Vec4::ZERO };
    let r = i.parts[pid].rd4(input_work(&slot, 1));
    r.truncate().extend(1.0)
}

/// `reset` (`0x5a6d60`): every point at the part's position in the sample space, spacings
/// (`+0x0c`, or random in [u16 `+0x1c`, `+0x1e`) when `+0x24`), one history entry, V = 1.
fn locus_reset(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let r = route(i, pid, s);
    let Some(mut l) = i.parts[pid].locus.get_mut(s).and_then(Option::take) else { return };
    (l.m, l.w) = locus_mats(i, pid, l.space);
    let at = l.m * r;
    let (lo, hi) = (pu16(&fep, p + 0x1c) as f32, pu16(&fep, p + 0x1e) as f32);
    // Not DDD's: world-space spacings in the caller's units (`Options::world_unit`).
    let unit = if l.space { i.world_unit } else { 1.0 };
    for k in 0..l.p.len() {
        l.p[k] = at;
        l.spacing[k] = unit * if pu8(&fep, p + 0x24) != 0 { lo + (hi - lo) * rng::unit(i.rng.as_mut()) } else { pf(&fep, p + 0x0c) };
    }
    let hc = l.hist.len();
    l.head = (hc - 1) % hc;
    l.hist[l.head] = l.p[0];
    l.v = 1;
    l.filled = 1;
    i.parts[pid].locus[s] = Some(l);
}

/// `update` (`0x5a62c0`): carry the stored points with the matrices' change (vt[4] `0x5a5390`,
/// unless `+0x28` or Lmatrix byte 1 & 7 is 3 or 4); then, when time runs, record the part's
/// position as the newest history entry and resample the history polyline from it at the
/// spacings (tails.md 1.4); the edges when `+0x25`.
fn locus_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let r = route(i, pid, s);
    let b91 = i.parts[pid].b91 & 7;
    let Some(mut l) = i.parts[pid].locus.get_mut(s).and_then(Option::take) else { return };
    let (m, w) = locus_mats(i, pid, l.space);
    if !(b91 == 3 || b91 == 4 || pu8(&fep, p + 0x28) == 1) {
        // D = inv(M_old) W_old inv(W_new) M_new (DDD order).
        let d = m * inverse(w) * l.w * inverse(l.m);
        for q in l.p.iter_mut().chain(l.hist.iter_mut()) {
            *q = d.transform_point3(q.truncate()).extend(q.w);
        }
    }
    (l.m, l.w) = (m, w);
    if i.dt != 0.0 {
        locus_record(&mut l, r);
        if pu8(&fep, p + 0x25) != 0 {
            let mut rm = l.m;
            rm.w_axis = Vec4::W;
            let a = rm.transform_vector3(Vec3::new(pf(&fep, p), -pf(&fep, p + 4), -pf(&fep, p + 8)));
            let b = rm.transform_vector3(Vec3::new(pf(&fep, p + 0x10), -pf(&fep, p + 0x14), -pf(&fep, p + 0x18)));
            for k in 0..l.p.len() {
                l.edge_a[k] = (l.p[k].truncate() + a).extend(1.0);
                l.edge_b[k] = (l.p[k].truncate() + b).extend(1.0);
            }
        }
    }
    i.parts[pid].locus[s] = Some(l);
}

/// The recording and resampling of `0x5a64c1..0x5a6886`.
fn locus_record(l: &mut Locus, r: Vec4) {
    let hc = l.hist.len();
    let n = l.p.len();
    let at = l.m * r;
    if l.first {
        l.p.iter_mut().for_each(|q| *q = at);
        l.hist[l.head] = at;
        l.first = false;
    }
    l.filled += 1;
    l.head = (l.head + hc - 1) % hc;
    l.hist[l.head] = at;
    l.p[0] = at;
    let nxt = (l.head + 1) % hc;
    let end = if l.filled > hc {
        l.filled = hc;
        (nxt + hc - 1) % hc
    } else {
        0
    };
    // Walk the history from the newest point, a point every spacing.
    let mut prev = l.hist[l.head];
    let mut k = nxt;
    let mut i = 1;
    while i < n {
        let sp = l.spacing[i - 1];
        let mut d = sp;
        loop {
            let seg = (l.hist[k].truncate() - prev.truncate()).length();
            if seg >= d {
                break;
            }
            d -= seg;
            prev = l.hist[k];
            k = (k + 1) % hc;
            if l.v < n && sp != 0.0 {
                l.progress = (sp - d) / sp;
            }
            if k == end {
                return;
            }
        }
        let dir = norm(l.hist[k].truncate() - prev.truncate());
        l.p[i] = (prev.truncate() + dir * d).extend(1.0);
        if i >= l.v {
            l.v += 1;
            l.progress = 1.0;
        }
        prev = l.p[i];
        if k == end {
            return;
        }
        i += 1;
    }
}

/// FecShapetail's state (work `+0` -> 0x20 bytes, tails.md 1.5): size factor and rotation per
/// point, the animation clock, the current key (-1 = over).
#[derive(Debug, Clone)]
pub(crate) struct Tail {
    size: Vec<f32>,
    rot: Vec<f32>,
    clock: f32,
    key: i32,
}

/// The ANM of a block at section-4 index `+off`, loaded (resident ones through resident.frr).
fn anm_at<'a>(fep: &'a Fep, resident: Option<&'a res::Resident>, p: usize, off: usize) -> Option<&'a [u8]> {
    let idx = pu32(fep, p + off) as i32;
    usize::try_from(idx).ok().and_then(|k| fep.anm.get(k)).and_then(|r| res::data(fep, resident, r))
}

/// The Locus object a Shapetail reads (input 2).
fn tail_locus(i: &Instance, pid: usize, s: usize) -> Option<&Locus> {
    let slot = i.def(pid)?.slots.get(s)?;
    i.parts[pid].locus.get(slot.inputs.get(2)?.slot as usize)?.as_deref()
}

/// Slot 0 (`0x5a24c0`): N = the Locus's N, sizes 1, rotations 0; with `+0x44` and the ANM, a
/// random start up to its length - 1.
fn tail_init(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let n = tail_locus(i, pid, s).map_or(0, |l| l.p.len());
    let mut clock = 0.0;
    if pu8(&fep, p + 0x44) != 0 {
        if let Some(anm) = anm_at(&fep, i.resident.as_deref(), p, 0x2c) {
            let len = u32::from_le_bytes(anm.get(0x14..0x18).and_then(|b| b.try_into().ok()).unwrap_or([0; 4])) as i32;
            clock = rng::unit(i.rng.as_mut()) * (len - 1) as f32;
        }
    }
    if let Some(t) = i.parts[pid].tail.get_mut(s) {
        *t = Some(Box::new(Tail { size: vec![1.0; n], rot: vec![0.0; n], clock, key: 0 }));
    }
}

/// Slot 1 (`0x5a2200`): per point, size `1 - rand01 x +0x3c` when `+0x45`, rotation `rand01 x
/// 180` when `+0x46` (INFERRED: all sizes, then all rotations: two loops).
fn tail_key(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let Some(mut t) = i.parts[pid].tail.get_mut(s).and_then(Option::take) else { return };
    for v in t.size.iter_mut() {
        *v = if pu8(&fep, p + 0x45) != 0 { 1.0 - rng::unit(i.rng.as_mut()) * pf(&fep, p + 0x3c) } else { 1.0 };
    }
    for v in t.rot.iter_mut() {
        *v = if pu8(&fep, p + 0x46) != 0 { rng::unit(i.rng.as_mut()) * 180.0 } else { 0.0 };
    }
    i.parts[pid].tail[s] = Some(t);
}

/// Slot 2 (`0x5a2390`): clock += dt; f = clock x speed (`+0x30`); past the ANM's length a
/// looping one wraps (`int(f) mod int(length)`), else key -1; the key whose summed durations
/// pass f. A looping ANM of length 0 (DDD: `idiv` by zero) ends.
fn tail_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let dt = i.dt;
    let resident = i.resident.clone();
    let Some(anm) = anm_at(&fep, resident.as_deref(), p, 0x2c) else { return };
    let Some(Some(t)) = i.parts[pid].tail.get_mut(s) else { return };
    let rd = |o: usize| anm.get(o..o + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    t.clock += dt;
    let speed = pf(&fep, p + 0x30);
    let mut f = t.clock * speed;
    let len = rd(0x14) as i32;
    if f >= len as f32 {
        if rd(0x10) & 0x80 == 0 || len == 0 {
            t.key = -1;
            return;
        }
        f = ((f as i32).wrapping_rem(len)) as f32;
        t.clock = f / speed;
    }
    let (keys, count) = (rd(0x2c) as usize, rd(0x1c) as usize);
    let mut sum = rd(keys + 8) as i32 as f32;
    let mut k = 0;
    while f >= sum && k + 1 < count.max(1) {
        k += 1;
        sum += rd(keys + 8 + 12 * k) as i32 as f32;
    }
    t.key = k as i32;
}

/// FecShapetail's draw (`0x5a2850`, tails.md 1.7) as data: nothing when the ANM is missing,
/// the animation is over or Lmatrix byte 1 is not 0 / 1. Per reached point: the colour lerped
/// head -> tail (`+0x00`, `+0x10`) times input 0 (/128) and clamped, the size factor lerped
/// (`+0x34`, `+0x38` percent) times the point's random size, its rotation.
pub(crate) fn tail_draw(i: &Instance, pid: usize, s: usize, p: usize) -> Option<crate::run::TailDraw> {
    let fep = &i.fep;
    anm_at(fep, i.resident.as_deref(), p, 0x2c)?;
    let t = i.parts[pid].tail.get(s)?.as_deref()?;
    let key = u16::try_from(t.key).ok()?;
    let l = tail_locus(i, pid, s)?;
    let slot = i.def(pid)?.slots.get(s)?;
    let part = &i.parts[pid];
    let c = part.rd4(input_work(slot, 0));
    // Lmatrix (input 1): its current block's byte 1.
    let lm = slot.inputs.get(1).and_then(|inp| part.params.get(inp.slot as usize).copied().flatten()).map_or(0, |q| pu8(fep, q + 1));
    if lm > 1 {
        return None;
    }
    let v = l.v.min(l.p.len()).max(1);
    let mut head = c * pv4(fep, p) / 128.0;
    let tail = c * pv4(fep, p + 0x10) / 128.0;
    let dcol = (tail - head) / v as f32;
    let (sh, st) = (pf(fep, p + 0x34), pf(fep, p + 0x38));
    let mut sz = sh * PERCENT;
    let ds = (st - sh) / v as f32 * PERCENT;
    let mut points = Vec::with_capacity(v);
    for k in 0..v {
        head = clamp255(head);
        points.push(crate::run::TailPoint {
            position: l.p[k].truncate(),
            scale: sz * t.size.get(k).copied().unwrap_or(1.0),
            angle: t.rot.get(k).copied().unwrap_or(0.0),
            colour: head,
        });
        head += dcol;
        sz += ds;
    }
    let b = pu8(fep, p + 0x48);
    Some(crate::run::TailDraw {
        anm_frame: key,
        world: l.space,
        facing: lm == 1,
        blend: if b == 0 { 4 } else { b },
        orient: (pu8(fep, p + 0x47) != 0).then(|| pv3(fep, p + 0x20)),
        reached: l.v,
        points,
    })
}

/// The points a Locus slot holds (P[0..N], newest first) and how many are reached.
pub(crate) fn locus_points(i: &Instance, pid: usize, s: usize) -> Option<(Vec<Vec3>, usize)> {
    let l = i.parts[pid].locus.get(s)?.as_deref()?;
    Some((l.p.iter().map(|q| q.truncate()).collect(), l.v))
}

// --- the ribbon: FecWave_bindlocus_strategy (106), FecBindlocus_polygon (105) (tails.md 2) ---

/// The WaveStrategy object (`0x5a0570`, 0xd0 bytes; tails.md 2.2), kept beside the part (DDD:
/// its pointer at work `+0`). Matrices in glam's form.
#[derive(Debug, Clone)]
pub(crate) struct Wave {
    /// `+0x00` M = local x F, `+0x40` W = world x F (or M).
    m: Mat4,
    w: Mat4,
    /// `+0x80` the wave offset, `+0x90` its target.
    cur: Vec3,
    tgt: Vec3,
    /// `+0xa0` / `+0xb0` the edge samples ([0] newest), `+0xa8` / `+0xb8` their velocities.
    a: Vec<Vec4>,
    va: Vec<Vec4>,
    b: Vec<Vec4>,
    vb: Vec<Vec4>,
    /// `+0xc0` reset request, `+0xc4` re-target countdown.
    reset: bool,
    countdown: u32,
}

/// `0x5a0940`: M = local x F, W = world x F (or M when the part has no world matrix); F =
/// `frame_of`.
fn wave_mats(i: &Instance, pid: usize) -> (Mat4, Mat4) {
    let part = &i.parts[pid];
    let f = frame_of(i, pid);
    let m = f * part.local.unwrap_or(Mat4::IDENTITY);
    (m, part.world.map_or(m, |w| f * w))
}

/// A point of the part's space (y, z negated) through M.
fn wave_point(fep: &Fep, at: usize, m: Mat4) -> Vec4 {
    let v = pv3(fep, at);
    m.transform_point3(Vec3::new(v.x, -v.y, -v.z)).extend(1.0)
}

/// Slot 0 (`0x5a82c0`): construct, `init` (`0x5a1cb0`): N = s16 `+0x0c`, zero arrays, reset
/// requested, countdown 1.
fn wave_init(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let n = pi16(&i.fep, p + 0x0c).max(0) as usize;
    let w = Wave {
        m: Mat4::IDENTITY,
        w: Mat4::IDENTITY,
        cur: Vec3::ZERO,
        tgt: Vec3::ZERO,
        a: vec![Vec4::ZERO; n],
        va: vec![Vec4::ZERO; n],
        b: vec![Vec4::ZERO; n],
        vb: vec![Vec4::ZERO; n],
        reset: true,
        countdown: 1,
    };
    if let Some(slot) = i.parts[pid].wave.get_mut(s) {
        *slot = Some(Box::new(w));
    }
}

/// `update` (`0x5a1060`, tails.md 2.3): the first one after init lays every sample at the part's
/// two points (`reset` `0x5a0a60`: sample k moved by k x drift); later ones carry the samples by
/// the frame delta (unless `+0x0f` or Lmatrix byte 1 & 7 is 3 or 4), then, when time runs, age
/// every sample by its velocity, record the two points as the newest, and move the wave. Once
/// per update, not scaled by dt (tails.md 7: DDD PC updates at 60 Hz, dt 0.5).
fn wave_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let b91 = i.parts[pid].b91 & 7;
    let (m, w) = wave_mats(i, pid);
    let Some(mut l) = i.parts[pid].wave.get_mut(s).and_then(Option::take) else { return };
    let drift = pv3(&fep, p + 0x20);
    let d = Vec3::new(drift.x, -drift.y, -drift.z);
    if l.reset {
        l.reset = false;
        (l.m, l.w) = (m, w);
        let (a0, b0) = (wave_point(&fep, p, m), wave_point(&fep, p + 0x10, m));
        for k in 0..l.a.len() {
            let kd = d * k as f32;
            l.a[k] = (a0.truncate() + kd).extend(1.0);
            l.b[k] = (b0.truncate() + kd).extend(1.0);
            l.va[k] = d.extend(0.0);
            l.vb[k] = d.extend(0.0);
        }
        i.parts[pid].wave[s] = Some(l);
        return;
    }
    if !(b91 == 3 || b91 == 4 || pu8(&fep, p + 0x0f) == 1) {
        // D = inv(M_old) W_old inv(W_new) M_new (DDD order); the velocities by its 3x3.
        let dm = m * inverse(w) * l.w * inverse(l.m);
        for q in l.a.iter_mut().chain(l.b.iter_mut()) {
            *q = dm.transform_point3(q.truncate()).extend(q.w);
        }
        for q in l.va.iter_mut().chain(l.vb.iter_mut()) {
            *q = dm.transform_vector3(q.truncate()).extend(q.w);
        }
    }
    (l.m, l.w) = (m, w);
    let n = l.a.len();
    if i.dt != 0.0 && n > 0 {
        for k in (1..n).rev() {
            l.a[k] = (l.a[k - 1].truncate() + l.va[k - 1].truncate()).extend(1.0);
            l.va[k] = l.va[k - 1];
            l.b[k] = (l.b[k - 1].truncate() + l.vb[k - 1].truncate()).extend(1.0);
            l.vb[k] = l.vb[k - 1];
        }
        l.a[0] = wave_point(&fep, p, m);
        l.b[0] = wave_point(&fep, p + 0x10, m);
        l.countdown = l.countdown.wrapping_sub(1);
        if l.countdown == 0 {
            l.countdown = pu8(&fep, p + 0x0e) as u32;
            let amp = pv3(&fep, p + 0x40);
            let x = sym(i.rng.as_mut()) * amp.x;
            let y = sym(i.rng.as_mut()) * amp.y;
            let z = sym(i.rng.as_mut()) * amp.z;
            l.tgt = Vec3::new(x, y, z);
        }
        l.cur += (l.tgt - l.cur) * pv3(&fep, p + 0x30);
        let wv = m.transform_vector3(l.cur);
        let v0 = Vec4::new(wv.x + drift.x, wv.y - drift.y, wv.z - drift.z, 0.0);
        l.va[0] = v0;
        l.vb[0] = v0;
    }
    i.parts[pid].wave[s] = Some(l);
}

/// The B-spline object (`0x59ef10` init, `0x59e730` eval; tails.md 2.6).
#[derive(Debug, Clone)]
struct Spline {
    n: usize,
    step: f32,
    count: usize,
}

impl Spline {
    /// `0x59ef10`: step = (N + 2) / S; count = the steps from t = -1 while N >= t (float
    /// accumulation; capped here).
    fn new(n: usize, sub: u8) -> Spline {
        let step = (n as f32 + 2.0) / sub as f32;
        let (mut t, mut count) = (-1.0f32, 0usize);
        loop {
            t += step;
            count += 1;
            if !(n as f32 >= t) || count >= 1 << 16 {
                break;
            }
        }
        Spline { n, step, count: if n == 0 { 0 } else { count } }
    }

    /// `0x59e730`: out(t) = sum over j = -2 .. N + 1 of b(t - j) P[clamp(j, 0, N - 1)], t from
    /// -1 by step; xyz (w 1 here).
    fn eval(&self, pts: &[Vec4]) -> Vec<Vec4> {
        fn b(x: f32) -> f32 {
            let x = x.abs();
            if x < 1.0 {
                (3.0 * x * x * x - 6.0 * x * x + 4.0) / 6.0
            } else if x < 2.0 {
                (2.0 - x) * (2.0 - x) * (2.0 - x) / 6.0
            } else {
                0.0
            }
        }
        let n = self.n.min(pts.len());
        let mut out = Vec::with_capacity(self.count);
        let mut t = -1.0f32;
        for _ in 0..self.count {
            let mut q = Vec3::ZERO;
            if n > 0 {
                for j in -2..=(n as i32 + 1) {
                    q += b(t - j as f32) * pts[j.clamp(0, n as i32 - 1) as usize].truncate();
                }
            }
            out.push(q.extend(1.0));
            t += self.step;
        }
        out
    }
}

/// FecBindlocus_polygon's state (work `+0x00` vertex buffer, `+0x04` spline, `+0x10..` U, V,
/// extents in texels; tails.md 2.4).
#[derive(Debug, Clone)]
pub(crate) struct Ribbon {
    /// `None`: no smoothing (`+0x52` = 0) or the cubic kind (not decoded, deviation 19).
    spline: Option<Spline>,
    /// The spline kind is neither 0 nor 1: DDD's spawn fails; here nothing is drawn.
    failed: bool,
    u: f32,
    v: f32,
    uext: f32,
    vext: f32,
    verts: Vec<crate::run::RibbonVertex>,
}

/// The Wave object a Bindlocus reads (input 2).
fn ribbon_wave(i: &Instance, pid: usize, s: usize) -> Option<&Wave> {
    let slot = i.def(pid)?.slots.get(s)?;
    i.parts[pid].wave.get(slot.inputs.get(2)?.slot as usize)?.as_deref()
}

/// A block's texture size (TEX `+0x1c` width, `+0x20` height; textures.md 3), if the section-2
/// record `+0x2c` has its bytes.
fn tex_size(fep: &Fep, resident: Option<&res::Resident>, p: usize) -> Option<(f32, f32)> {
    let idx = pu32(fep, p + 0x2c) as i32;
    let d = usize::try_from(idx).ok().and_then(|k| fep.tex.get(k)).and_then(|r| res::data(fep, resident, r))?;
    let rd = |o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    Some((rd(0x1c)? as f32, rd(0x20)? as f32))
}

/// `uv_init` (`0x59e4a0`; slot 1's `0x59d2a0` without the randoms): only when textured and the
/// texture exists. Random start `rand01 x (extent - offset)` (`0x58c630`, INFERRED).
fn ribbon_uv(i: &mut Instance, r: &mut Ribbon, p: usize, randomize: bool) {
    let fep = i.fep.clone();
    if pu8(&fep, p + 0x48) == 0 || tex_size(&fep, i.resident.as_deref(), p).is_none() {
        return;
    }
    r.uext = pu16(&fep, p + 0x44) as f32;
    r.vext = pu16(&fep, p + 0x46) as f32;
    let (uoff, voff) = (pu16(&fep, p + 0x40) as f32, pu16(&fep, p + 0x42) as f32);
    if pu8(&fep, p + 0x49) == 0 {
        r.u = uoff;
    } else if randomize {
        r.u = uoff + rng::unit(i.rng.as_mut()) * (r.uext - uoff);
    }
    if pu8(&fep, p + 0x4a) == 0 {
        r.v = voff;
    } else if randomize {
        r.v = voff + rng::unit(i.rng.as_mut()) * (r.vext - voff);
    }
}

/// Slot 0 (`0x59dcf0`): the UV start, the spline object (`+0x52`; kind `+0x53` 1 = B-spline with
/// subdivision `+0x54`), a zeroed vertex buffer of 2 x count.
fn ribbon_init(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let n = ribbon_wave(i, pid, s).map_or(0, |w| w.a.len());
    let mut r = Ribbon { spline: None, failed: false, u: 0.0, v: 0.0, uext: 0.0, vext: 0.0, verts: Vec::new() };
    ribbon_uv(i, &mut r, p, true);
    let mut count = n;
    if pu8(&fep, p + 0x52) != 0 {
        match pu8(&fep, p + 0x53) {
            1 => {
                let sp = Spline::new(n, pu8(&fep, p + 0x54));
                count = sp.count;
                r.spline = Some(sp);
            }
            0 => {}
            _ => r.failed = true,
        }
    }
    r.verts = vec![crate::run::RibbonVertex { position: Vec3::ZERO, uv: Vec2::ZERO, colour: [0; 4] }; 2 * count];
    if let Some(slot) = i.parts[pid].ribbon.get_mut(s) {
        *slot = Some(Box::new(r));
    }
}

/// Wrap into [0, n) by repeated -/+ n; absurd values are reduced first.
fn wrap_texels(mut x: f32, n: f32) -> f32 {
    if !(n > 0.0) || !x.is_finite() {
        return x;
    }
    if x.abs() > 1.0e6 {
        x = x.rem_euclid(n);
    }
    while x >= n {
        x -= n;
    }
    while x < 0.0 {
        x += n;
    }
    x
}

/// Slot 2 (`0x59d360`, tails.md 2.5): scroll U / V (`+0x30`, `+0x34`, x dt), take both edges
/// (resampled by the spline), and rebuild the strip: colour lerped head -> tail over the points
/// (x input 0 / 128, clamped), the head fade (`+0x50`, `+0x38` %), UVs by `+0x4c..+0x4e`.
fn ribbon_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let fep = i.fep.clone();
    let tex = tex_size(&fep, i.resident.as_deref(), p);
    let textured = pu8(&fep, p + 0x48) != 0;
    let dt = i.dt;
    let Some(slot) = slot_of(i, pid, s) else { return };
    // Input 0 x the element's colour (1 here: the caller's) / 128.
    let g = i.parts[pid].rd4(input_work(&slot, 0)) / 128.0;
    let Some(mut r) = i.parts[pid].ribbon.get_mut(s).and_then(Option::take) else { return };
    let (tw, th) = tex.unwrap_or((1.0, 1.0));
    if textured && tex.is_some() {
        r.u = wrap_texels(r.u - dt * pf(&fep, p + 0x30), tw);
        r.v = wrap_texels(r.v - dt * pf(&fep, p + 0x34), th);
    }
    let (pa, pb) = match ribbon_wave(i, pid, s) {
        Some(w) => match &r.spline {
            Some(sp) => (sp.eval(&w.a), sp.eval(&w.b)),
            None => (w.a.clone(), w.b.clone()),
        },
        None => (Vec::new(), Vec::new()),
    };
    let count = pa.len().min(pb.len()).min(r.verts.len() / 2);
    if count > 0 {
        let head = clamp255(pv4(&fep, p) * g);
        let tail = clamp255(pv4(&fep, p + 0x10) * g);
        let dcol = (tail - head) / count as f32;
        let (mut fstep, mut f) = (0.0f32, 1.0f32);
        if pu8(&fep, p + 0x50) != 0 {
            let k = (pf(&fep, p + 0x38) * 0.01 * (count - 1) as f32 + 1.0) as i32;
            if k > 1 {
                fstep = 1.0 / k as f32;
                f = 0.0;
            }
        }
        let (flip_u, flip_v) = (pu8(&fep, p + 0x4d) != 0, pu8(&fep, p + 0x4e) != 0);
        let (u, v, ue, ve, cn) = (r.u, r.v, r.uext, r.vext, count as f32);
        let (mut ua, mut ub, mut va, mut vb, du, dv);
        if pu8(&fep, p + 0x4c) == 0 {
            // U across, V along.
            (ua, ub) = if flip_u { ((u + ue) / tw, u / tw) } else { (u / tw, (u + ue) / tw) };
            du = 0.0;
            let (v0, d) = if flip_v { (v / th, ve / th / cn) } else { ((v + ve) / th, -ve / th / cn) };
            (va, vb, dv) = (v0, v0, d);
        } else {
            (vb, va) = if flip_v { ((v + ve) / th, v / th) } else { (v / th, (v + ve) / th) };
            dv = 0.0;
            let (u0, d) = if flip_u { (u / tw, ue / tw / cn) } else { ((u + ue) / tw, -ue / tw / cn) };
            (ua, ub, du) = (u0, u0, d);
        }
        let mut c = head;
        for k in 0..count {
            c = clamp255(c);
            let mut a = c.w;
            if f < 1.0 {
                a = (a * f).min(255.0);
            }
            f += fstep;
            // cvttss2si per byte.
            let rgba = [c.x as i32 as u8, c.y as i32 as u8, c.z as i32 as u8, a as i32 as u8];
            let (uv_a, uv_b) = if textured { (Vec2::new(ua, va), Vec2::new(ub, vb)) } else { (Vec2::ZERO, Vec2::ZERO) };
            r.verts[2 * k] = crate::run::RibbonVertex { position: pa[k].truncate(), uv: uv_a, colour: rgba };
            r.verts[2 * k + 1] = crate::run::RibbonVertex { position: pb[k].truncate(), uv: uv_b, colour: rgba };
            ua += du;
            va += dv;
            ub += du;
            vb += dv;
            c += dcol;
        }
    }
    i.parts[pid].ribbon[s] = Some(r);
}

/// FecBindlocus_polygon's draw (`0x59deb0`, tails.md 2.7) as data: nothing when textured and the
/// texture index is < 0 or missing.
pub(crate) fn ribbon_draw(i: &Instance, pid: usize, s: usize, p: usize) -> Option<crate::run::RibbonDraw> {
    let fep = &i.fep;
    let r = i.parts[pid].ribbon.get(s)?.as_deref()?;
    let textured = pu8(fep, p + 0x48) != 0;
    if r.failed || (textured && tex_size(fep, i.resident.as_deref(), p).is_none()) {
        return None;
    }
    Some(crate::run::RibbonDraw {
        vertices: r.verts.clone(),
        texture: textured.then(|| pu32(fep, p + 0x2c) as i32),
        blend: pu8(fep, p + 0x51),
        sampler: pu8(fep, p + 0x4b) != 0,
        depth: (pu8(fep, p + 0x55) != 0).then(|| pv3(fep, p + 0x20)),
    })
}

// --- line particles: FecEllipse_line_particle (104), FecRadialg_line_particle (100)
// (tails.md 4, 5) ---

/// One FecEllipse_line_particle particle (0xe0 bytes, tails.md 4.2).
#[derive(Debug, Clone, Copy, Default)]
struct EllParticle {
    /// `+0x00` position (part-local, before the axes).
    pos: Vec4,
    /// `+0x10` Rs (rotation per update), `+0x50` Ra (its acceleration), `+0x90` Ra's copy.
    rs: Mat4,
    ra: Mat4,
    ra_draw: Mat4,
    /// `+0xd0` life left, `+0xd2` fade start, `+0xd4` fade, `+0xd8` fade step, `+0xdc` alive.
    life: u16,
    half: u16,
    fade: f32,
    fstep: f32,
    alive: bool,
}

/// One FecRadialg_line_particle particle (0x38 bytes + its n history entries, tails.md 5.2).
#[derive(Debug, Clone, Default)]
struct RadParticle {
    vel: Vec4,
    acc: Vec4,
    life: u16,
    fade: f32,
    alive: bool,
    /// {colour (0..1), position} per history entry, [0] newest.
    hist: Vec<(Vec4, Vec4)>,
}

/// A line-particle pool (Ellipse: work `+0` pool, `+8` accumulator; RadialGLine `+0x10` pool,
/// `+0x1c` accumulator).
#[derive(Debug, Clone, Default)]
pub(crate) struct Lines {
    acc: f32,
    ell: Vec<EllParticle>,
    rad: Vec<RadParticle>,
}

/// The shared emission rule (`0x55fa6c..0x55fb1a`, `0x565126..0x56518a`; tails.md 4.5).
fn line_emit(dt: f32, rate: f32, interval: u8, acc: &mut f32) -> i32 {
    let mut x = dt * rate + *acc;
    if x < 1.0 {
        *acc = x;
        return 0;
    }
    if dt < 1.0 {
        x = rate;
    }
    let count = x as i32;
    *acc = x - count as f32 - rate * interval as f32;
    count
}

/// Adds / subtracts 2 pi until in [-pi, pi] (absurd values reduced first).
fn wrap_pi(mut x: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    if !x.is_finite() {
        return x;
    }
    if x.abs() > 1.0e4 {
        x %= TAU;
    }
    while x > PI {
        x -= TAU;
    }
    while x < -PI {
        x += TAU;
    }
    x
}

/// `[0x7b1208]`: radians / this = minus degrees (INFERRED -pi/180).
const NEG_RAD_PER_DEG: f32 = -0.017_453_292;
const RAD_PER_DEG: f32 = std::f32::consts::PI / 180.0;

/// `2 rand01 - 1`.
fn sym(rng: &mut dyn Rng) -> f32 {
    2.0 * rng::unit(rng) - 1.0
}

/// Slot 0 (`0x560070` Ellipse, `0x576610` RadialG): a pool of max particles (`+0x44` / `+0x3c`).
/// A life of 0 (`+0x46` / `+0x3e`) aborts DDD's spawn; here the pool is not made and nothing is
/// drawn (deviation 8).
fn lines_init(i: &mut Instance, pid: usize, s: usize, p: usize, id: u16) {
    let fep = i.fep.clone();
    let (max, life) = if id == 104 { (pu16(&fep, p + 0x44), pu16(&fep, p + 0x46)) } else { (pu16(&fep, p + 0x3c), pu16(&fep, p + 0x3e)) };
    if life == 0 {
        return;
    }
    let mut l = Lines::default();
    if id == 104 {
        l.ell = vec![EllParticle::default(); max as usize];
    } else {
        let n = pu8(&fep, p + 0x42) as usize;
        l.rad = vec![RadParticle { hist: vec![(Vec4::ZERO, Vec4::ZERO); n], ..Default::default() }; max as usize];
    }
    if let Some(slot) = i.parts[pid].lines.get_mut(s) {
        *slot = Some(Box::new(l));
    }
}

/// FecEllipse_line_particle slot 2 (`0x55fa20`, tails.md 4.3): emission (each new particle in
/// the first free slot at or after the last one taken this update, INFERRED), then every live
/// particle in slot order: life - 1 (dies at 0), the fade over the second half of its life, its
/// position turned by Rs, then Rs by Ra. Per update, not scaled by dt.
fn ellipse_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    if i.dt == 0.0 {
        return;
    }
    let fep = i.fep.clone();
    let Some(mut l) = i.parts[pid].lines.get_mut(s).and_then(Option::take) else { return };
    let count = line_emit(i.dt, pf(&fep, p + 0x40), pu8(&fep, p + 0x4b), &mut l.acc);
    let life_p = pu16(&fep, p + 0x46);
    let mut from = 0;
    for _ in 0..count.max(0) {
        let Some(k) = (from..l.ell.len()).find(|&k| !l.ell[k].alive) else { break };
        from = k + 1;
        let rng = i.rng.as_mut();
        let q = &mut l.ell[k];
        let d = Vec3::new(sym(rng), sym(rng), sym(rng));
        let len = d.length_squared();
        let len = if len < 0.0 { 0.001 } else { len.sqrt() };
        let d = if len == 0.0 { Vec3::ZERO } else { d / len };
        let r = rng::unit(rng) * pf(&fep, p + 0x3c);
        q.pos = d.extend(q.pos.w) * r;
        let w = wrap_pi((pf(&fep, p + 0x0c) + sym(rng) * pf(&fep, p + 0x30)) * RAD_PER_DEG);
        q.rs = rot(Axis::Y, w / NEG_RAD_PER_DEG);
        let al = wrap_pi((pf(&fep, p + 0x34) + sym(rng) * pf(&fep, p + 0x38)) * RAD_PER_DEG);
        q.ra = rot(Axis::Y, al / NEG_RAD_PER_DEG);
        q.ra_draw = q.ra;
        q.life = ((sym(rng) * pu16(&fep, p + 0x48) as f32) as i32 + life_p as i32) as u16;
        q.half = life_p >> 1;
        q.fade = 1.0;
        q.fstep = 2.0 / life_p as f32;
        q.alive = true;
    }
    for q in l.ell.iter_mut().filter(|q| q.alive) {
        q.life = q.life.wrapping_sub(1);
        if q.life == 0 {
            q.alive = false;
            continue;
        }
        if q.life < q.half {
            q.fade = (q.fade - q.fstep).max(0.0);
        }
        q.pos = q.rs.transform_point3(q.pos.truncate()).extend(1.0);
        // DDD `Rs x Ra` (Rs applied first).
        q.rs = q.ra * q.rs;
    }
    i.parts[pid].lines[s] = Some(l);
}

/// FecRadialg_line_particle slot 2 (`0x5765b0` -> `0x565870`, tails.md 5.3): emission into a
/// cone of half-angle `+0x38` / 2 about local +Y, then every live particle: life - 1, the
/// history shifted, the newest moved by the velocity, the velocity by the acceleration and
/// gravity, the fade over the second half. Per update, not scaled by dt.
fn radial_update(i: &mut Instance, pid: usize, s: usize, p: usize) {
    if i.dt == 0.0 {
        return;
    }
    let fep = i.fep.clone();
    let Some(mut l) = i.parts[pid].lines.get_mut(s).and_then(Option::take) else { return };
    let count = line_emit(i.dt, pf(&fep, p + 0x34), pu8(&fep, p + 0x43), &mut l.acc);
    let life_p = pu16(&fep, p + 0x3e);
    let (head, tail) = (pv4(&fep, p), pv4(&fep, p + 0x10));
    let mut from = 0;
    for _ in 0..count.max(0) {
        let Some(k) = (from..l.rad.len()).find(|&k| !l.rad[k].alive) else { break };
        from = k + 1;
        let rng = i.rng.as_mut();
        let q = &mut l.rad[k];
        let h = wrap_pi(pf(&fep, p + 0x38) * RAD_PER_DEG * 0.5);
        let a1 = sym(rng) * h;
        let a2 = sym(rng) * h;
        // (0, 1, 0, 1) x RotZ(a1) x RotX(a2) (DDD order).
        let d = rot(Axis::X, a2 / RAD_PER_DEG) * rot(Axis::Z, a1 / RAD_PER_DEG) * Vec4::new(0.0, 1.0, 0.0, 1.0);
        q.acc = d * (pf(&fep, p + 0x28) + sym(rng) * pf(&fep, p + 0x2c));
        q.vel = d * (pf(&fep, p + 0x20) + sym(rng) * pf(&fep, p + 0x24));
        let p0 = (q.vel.truncate() * rng::unit(rng)).extend(1.0);
        q.life = ((sym(rng) * pu16(&fep, p + 0x40) as f32) as i32 + life_p as i32) as u16;
        q.fade = 1.0;
        q.alive = true;
        let n = q.hist.len();
        for (k, e) in q.hist.iter_mut().enumerate() {
            *e = (head / 128.0 + k as f32 * (tail - head) / (128.0 * n as f32), p0);
        }
    }
    let half = life_p >> 1;
    let g = pf(&fep, p + 0x30);
    for q in l.rad.iter_mut().filter(|q| q.alive) {
        q.life = q.life.wrapping_sub(1);
        if q.life == 0 {
            q.alive = false;
            continue;
        }
        for k in (1..q.hist.len()).rev() {
            q.hist[k].1 = q.hist[k - 1].1;
        }
        if let Some(h0) = q.hist.first_mut() {
            h0.1 = (h0.1.truncate() + q.vel.truncate()).extend(h0.1.w);
        }
        q.vel += q.acc;
        q.vel.y -= g;
        if q.life < half {
            q.fade = (q.fade - 1.0 / half as f32).max(0.0);
        }
    }
    i.parts[pid].lines[s] = Some(l);
}

/// `min(1, max(0, v)) x 255` as bytes (INFERRED truncation).
fn bytes01(v: Vec4) -> [u8; 4] {
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
    [b(v.x), b(v.y), b(v.z), b(v.w)]
}

/// The line draws (`0x560130` Ellipse, tails.md 4.4; `0x576730` -> `0x563c80` RadialG, 5.4) as
/// data: per live particle a strip of n points with its colours. C = input 0 clamped / 255 (the
/// element's colour is the caller's).
pub(crate) fn lines_draw(i: &Instance, pid: usize, s: usize, p: usize) -> Option<crate::run::LineDraw> {
    let fep = &i.fep;
    let l = i.parts[pid].lines.get(s)?.as_deref()?;
    let slot = i.def(pid)?.slots.get(s)?;
    let part = &i.parts[pid];
    let c = clamp255(part.rd4(input_work(slot, 0))) / 255.0;
    let lm = slot.inputs.get(1).and_then(|inp| part.params.get(inp.slot as usize).copied().flatten()).map_or(0, |q| pu8(fep, q + 1));
    let local = part.local.unwrap_or(Mat4::IDENTITY);
    let mut strips = Vec::new();
    let (matrix, blend);
    if slot.id == 104 {
        let n = pu8(fep, p + 0x4a) as usize;
        let (head, tail) = (pv4(fep, p + 0x10), pv4(fep, p + 0x20));
        let dcol = (head - tail) / (128.0 * n as f32);
        for q in l.ell.iter().filter(|q| q.alive) {
            let am = c.w * q.fade;
            let (mut qm, mut t) = (inverse(q.rs), Mat4::IDENTITY);
            let mut col = head / 128.0;
            let mut strip = Vec::with_capacity(n);
            for _ in 0..n {
                let pos = t.transform_point3(q.pos.truncate());
                // DDD `T = T x Q; Q = Q x Ra`.
                t = qm * t;
                qm = q.ra_draw * qm;
                strip.push(crate::run::LineVertex { position: pos, colour: bytes01((col.truncate() * c.truncate()).extend(col.w * am)) });
                col -= dcol;
            }
            strips.push(strip);
        }
        // DDD `diag(axes) x local`.
        matrix = local * Mat4::from_diagonal(pv3(fep, p).extend(1.0));
        blend = pu8(fep, p + 0x4c);
    } else {
        for q in l.rad.iter().filter(|q| q.alive) {
            strips.push(
                q.hist
                    .iter()
                    .map(|(col, pos)| crate::run::LineVertex { position: pos.truncate(), colour: bytes01((col.truncate() * c.truncate()).extend(col.w * c.w * q.fade)) })
                    .collect(),
            );
        }
        matrix = local;
        blend = pu8(fep, p + 0x44);
    }
    Some(crate::run::LineDraw { strips, matrix, facing: lm, blend })
}

// --- FecFlash (107): a screen-space rectangle (aura.md 5) ---

/// DDD's virtual screen (`[0xaa6e28]` / `[0xaa6e2c]` static 400 / 240; the PC rewrites them for
/// its aspect, `0x60ef75`: not modelled, a drawer maps this screen onto its own) and its centre
/// (`[0x9e87d4]` / `[0x9e87dc]`).
pub const SCREEN: (i32, i32) = (400, 240);
const SCREEN_CENTRE: (i32, i32) = (200, 120);

/// FecFlash's two allocations (`0x596cb0`): A `{f32 value, f32 base, B*}`, B six s16 `{x0, y0,
/// 0, x1, y1, 0}`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Flash {
    value: f32,
    base: f32,
    rect: [i16; 4],
}

/// Slot 2 (`0x596be0`): the scalar, then the rectangle (the rect when `+0x19`, clipped to the
/// screen by unsigned compares; else the whole screen).
fn flash_update(i: &mut Instance, pid: usize, s: usize, p: usize, t: f32) {
    let fep = i.fep.clone();
    let Some(Some(f)) = i.parts[pid].flash.get_mut(s) else { return };
    f.value = f.base + t * pf(&fep, p + 0x14);
    let (sw, sh) = SCREEN;
    f.rect = if pu8(&fep, p + 0x19) != 0 {
        let (x, y, w, h) = (pu32(&fep, p), pu32(&fep, p + 4), pu32(&fep, p + 8), pu32(&fep, p + 12));
        let x1 = if x.wrapping_add(w) > sw as u32 { sw } else { x.wrapping_add(w) as i32 };
        let y1 = if y.wrapping_add(h) > sh as u32 { sh } else { y.wrapping_add(h) as i32 };
        [x as i16, y as i16, x1 as i16, y1 as i16]
    } else {
        // `200 - half width`, `120 - half height`: the screen's top-left corner.
        [(SCREEN_CENTRE.0 - sw / 2) as i16, (SCREEN_CENTRE.1 - sh / 2) as i16, sw as i16, sh as i16]
    };
}

/// FecFlash's draw (`0x596da0`): the rectangle, its colour input and blend selector.
pub(crate) fn flash_draw(i: &Instance, pid: usize, s: usize, p: usize) -> Option<crate::run::FlashDraw> {
    let f = (*i.parts[pid].flash.get(s)?)?;
    let slot = slot_of(i, pid, s)?;
    Some(crate::run::FlashDraw {
        rect: f.rect,
        full_screen: pu8(&i.fep, p + 0x19) == 0,
        colour: i.parts[pid].rd4(input_work(&slot, 0)),
        blend: pu8(&i.fep, p + 0x18),
        value: f.value,
    })
}

// --- dispatch by vtable slot ---

fn ids(i: &Instance, pid: usize, s: usize) -> Option<(u16, usize)> {
    let d = i.def(pid)?;
    let slot = d.slots.get(s)?;
    Some((slot.id, slot.work_offset()))
}

/// Slot 0. False aborts the spawn (no class here fails).
pub(crate) fn init(i: &mut Instance, pid: usize, s: usize, p: usize) -> bool {
    let Some((id, at)) = ids(i, pid, s) else { return true };
    if let Some((fam, form)) = family(id) {
        fam_init(i, pid, at, p, fam, form);
        return true;
    }
    match id {
        14 => lmatrix(i, pid, s, p, false),
        18 => i.parts[pid].wr(at, 0.0),
        // FecColorblink_parameter `0x5852c0`; FecColorblink `0x584c70`: a copy of input 0.
        61 => blink_reset(i, pid, at, p),
        62 => {
            if let Some(slot) = slot_of(i, pid, s) {
                let v = i.parts[pid].rd4(input_work(&slot, 0));
                i.parts[pid].wr4(at, v);
            }
        }
        68 => locus_init(i, pid, s, p),
        // FecNull_routegenerator `0x5853d0`: work = `[0xaa3df0]`, INFERRED (0, 0, 0, 0).
        79 => i.parts[pid].wr4(at, Vec4::ZERO),
        80 => tail_init(i, pid, s, p),
        100 | 104 => lines_init(i, pid, s, p, id),
        105 => ribbon_init(i, pid, s, p),
        106 => wave_init(i, pid, s, p),
        // FecFlash `0x596cb0`: value = base = v0, the rectangle zeroed.
        107 => {
            let v0 = pf(&i.fep, p + 0x10);
            if let Some(f) = i.parts[pid].flash.get_mut(s) {
                *f = Some(Flash { value: v0, base: v0, rect: [0; 4] });
            }
        }
        150 => {
            i.parts[pid].wr_u16(at + VTX_CURSOR, 0);
            i.parts[pid].wr_u8(at + VTX_PENDING, 0);
        }
        151 => i.parts[pid].wr_u8(at, 0),
        152 => anm_init(i, pid, at, p),
        153 => mega_init(i, pid, s, at, p),
        155 => mdlts_init(i, pid, at, p),
        _ => {}
    }
    true
}

/// Slot 1.
pub(crate) fn key(i: &mut Instance, pid: usize, s: usize, new: usize, old: Option<usize>, t: f32) {
    let Some((id, at)) = ids(i, pid, s) else { return };
    if let Some((fam, form)) = family(id) {
        return fam_key(i, pid, at, fam, form, new, old, t);
    }
    match id {
        // FecGenerate `0x5819f0`: acc += old rate if another key list already ran in this call.
        18 => {
            let v = if i.keys_run > 0 { i.parts[pid].rd(at) + old.map_or(0.0, |o| pf(&i.fep, o)) } else { 0.0 };
            i.parts[pid].wr(at, v);
        }
        // FecGravity `0x585350`: input 0's acceleration.y += s16 g / 100 x 0.098.
        19 => {
            let Some(slot) = slot_of(i, pid, s) else { return };
            let w = input_work(&slot, 0) + ACC + 4;
            let g = pi16(&i.fep, new) as f32 / 100.0 * GRAVITY;
            let part = &mut i.parts[pid];
            part.wr(w, g + part.rd(w));
        }
        22..=25 => rnd(i, pid, s, id, new),
        // FecColorblink_parameter `0x584ff0`: reset with the new block; FecColorblink `0x584af0`.
        61 => blink_reset(i, pid, at, new),
        62 => {
            if let Some(slot) = slot_of(i, pid, s) {
                let v = i.parts[pid].rd4(input_work(&slot, 0));
                i.parts[pid].wr4(at, v);
            }
        }
        // FecLocus_tail_strategy `0x5a20f0`: nothing.
        80 => tail_key(i, pid, s, new),
        // FecBindlocus_polygon `0x59d2a0`: uv_init without the randoms. The line particles
        // (`0x55fa10`, `0x5765a0`) and Wave_bindlocus_strategy (`0x5a82a0`) do nothing.
        105 => {
            if let Some(mut r) = i.parts[pid].ribbon.get_mut(s).and_then(Option::take) {
                ribbon_uv(i, &mut r, new, false);
                i.parts[pid].ribbon[s] = Some(r);
            }
        }
        // FecFlash `0x596bb0`: base += new v0 - old v0 (the span x old rate terms cancel).
        107 => {
            let Some(o) = old else { return };
            let d = pf(&i.fep, new + 0x10) - pf(&i.fep, o + 0x10);
            if let Some(Some(f)) = i.parts[pid].flash.get_mut(s) {
                f.base += d;
            }
        }
        // FecGenerateanchorvertex `0x57faf0`: as FecGenerateanchor.
        150 => {
            if i.parts[pid].rd_u8(at + VTX_PENDING) != 0 {
                if let Some(o) = old {
                    vertex_fire(i, pid, at, o);
                }
            }
            i.parts[pid].wr_u8(at + VTX_PENDING, 1);
        }
        // FecGenerateanchor `0x5802d0`: a pending burst fires with the old block; then pending.
        151 => {
            if i.parts[pid].rd_u8(at) != 0 {
                if let Some(o) = old {
                    anchor_fire(i, pid, s, o);
                }
            }
            i.parts[pid].wr_u8(at, 1);
        }
        153 => mega_key(i, pid, s, at, new),
        155 => mdlts_key(i, pid, at, new, old, t),
        _ => {}
    }
}

/// Slot 2.
pub(crate) fn update(i: &mut Instance, pid: usize, s: usize, p: usize, t: f32) {
    let Some((id, at)) = ids(i, pid, s) else { return };
    if let Some((fam, form)) = family(id) {
        return fam_update(i, pid, at, fam, form, t);
    }
    match id {
        14 => lmatrix(i, pid, s, p, true),
        18 => generate(i, pid, s, at, p, t),
        61 => blink_update(i, pid, at, p),
        62 => colorblink(i, pid, s),
        68 => locus_update(i, pid, s, p),
        80 => tail_update(i, pid, s, p),
        100 => radial_update(i, pid, s, p),
        104 => ellipse_update(i, pid, s, p),
        105 => ribbon_update(i, pid, s, p),
        106 => wave_update(i, pid, s, p),
        107 => flash_update(i, pid, s, p, t),
        // FecGenerateanchorvertex `0x57fb30`.
        150 => {
            if i.parts[pid].rd_u8(at + VTX_PENDING) != 0 {
                vertex_fire(i, pid, at, p);
                i.parts[pid].wr_u8(at + VTX_PENDING, 0);
            }
        }
        // FecGenerateanchor `0x580310`: a pending burst fires with the current block.
        151 => {
            if i.parts[pid].rd_u8(at) != 0 {
                anchor_fire(i, pid, s, p);
                i.parts[pid].wr_u8(at, 0);
            }
        }
        152 => anm_update(i, pid, at, p),
        153 => mega_update(i, pid, s, at, p),
        _ => {}
    }
}

/// Slot 4: FecMegaanm_plus frees its buffer (`0x58dfe0`), FecLocus_tail_strategy its object
/// (`0x5a21b0`), FecShapetail its arrays (`0x5a27d0`), Wave_bindlocus_strategy its object
/// (`0x5a8320`), Bindlocus_polygon its buffer and spline (`0x59de50`), the line particles their
/// pools (`0x560100`, `0x576710`), FecFlash its two blocks (`0x596d40`); nothing else to free here.
pub(crate) fn release(i: &mut Instance, pid: usize, s: usize, _p: usize) {
    let part = &mut i.parts[pid];
    if let Some(f) = part.flash.get_mut(s) {
        *f = None;
    }
    if let Some(m) = part.mega.get_mut(s) {
        *m = None;
    }
    if let Some(l) = part.locus.get_mut(s) {
        *l = None;
    }
    if let Some(t) = part.tail.get_mut(s) {
        *t = None;
    }
    if let Some(w) = part.wave.get_mut(s) {
        *w = None;
    }
    if let Some(r) = part.ribbon.get_mut(s) {
        *r = None;
    }
    if let Some(l) = part.lines.get_mut(s) {
        *l = None;
    }
}

/// Slot 5. FecMegaanm_plus (`0x58e7f0`) returns 1: its buffer, accumulator and key clock carry
/// over a loop.
pub(crate) fn rewind(i: &mut Instance, pid: usize, s: usize, new: usize, t_new: f32, old: Option<usize>, t_old: f32) {
    let Some((id, at)) = ids(i, pid, s) else { return };
    if let Some((fam, form)) = family(id) {
        return fam_rewind(i, pid, at, fam, form, new, t_new, old, t_old);
    }
    match id {
        // FecGenerateanchor `0x580890`: pending.
        151 => i.parts[pid].wr_u8(at, 1),
        // FecLocus_tail_strategy `0x5a21d0`: reset when `+0x27` (0 in every file).
        68 => {
            if pu8(&i.fep, new + 0x27) != 0 {
                locus_reset(i, pid, s, new);
            }
        }
        // FecGenerateanchorvertex `0x5802c0`: pending.
        150 => i.parts[pid].wr_u8(at + VTX_PENDING, 1),
        // FecFlash `0x597160`: base += t_old x old rate - t_new x new rate, then its update.
        107 => {
            let fep = i.fep.clone();
            let old_rate = old.map_or(0.0, |o| pf(&fep, o + 0x14));
            if let Some(Some(f)) = i.parts[pid].flash.get_mut(s) {
                f.base += t_old * old_rate - t_new * pf(&fep, new + 0x14);
            }
            flash_update(i, pid, s, new, t_new);
        }
        155 => mdlts_rewind(i, pid, at, new, t_new, old, t_old),
        _ => {}
    }
}

/// Slot 6 (first update).
pub(crate) fn first(i: &mut Instance, pid: usize, s: usize, p: usize) {
    let Some((id, at)) = ids(i, pid, s) else { return };
    match id {
        1..=3 => pos_first(i, pid, at),
        // FecLmatrix `0x582d30`: a root part captures its frame (`0x5b85a0`, mode = byte 0).
        14 => {
            if i.parts[pid].parent.is_none() {
                let mode = pu8(&i.fep, p);
                i.capture_frame(pid, None, mode);
            }
        }
        _ => {}
    }
}

/// Slot 7 (after the pass-1 updates): no class here overrides it.
pub(crate) fn after(_i: &mut Instance, _pid: usize, _s: usize, _p: usize, _t: f32) {}

/// Is a slot of this class one of the Col family (its output = a colour)?
pub(crate) fn is_colour(id: u16) -> bool {
    matches!(id, 15..=17)
}

/// Scale family.
pub(crate) fn is_scale(id: u16) -> bool {
    matches!(id, 7..=9)
}

/// Position family.
pub(crate) fn is_position(id: u16) -> bool {
    matches!(id, 1..=3)
}
