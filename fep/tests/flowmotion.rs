//! Tests on DDD's own files (D:/KH-Extract/ddd/original). Each test prints a message and passes
//! when the files are not there.

use std::path::Path;
use std::sync::Arc;

use fep::{Fep, Instance, Resident};

const ROOT: &str = "D:/KH-Extract/ddd/original/effect";
const DUMPS: &str = "D:/KH-Extract/ddd/fep_dump";

/// The seven Flowmotion FEPs (docs/fep/README.md "Labels").
const FILES: [&str; 7] = [
    "sys/ffa/bin/p_fa_ajc0.fep",
    "sys/ffa/bin/p_fa_akd0.fep",
    "sys/ffa/bin/p_fa_ask0.fep",
    "sys/ffa/bin/p_fa_com0.fep",
    "sys/pcm/bin/p_ac_com0.fep",
    "pc/ac/bin/p_fa_bos_0.fep",
    "wep/ex/bin/w_010_so.fep",
];

/// The 19 Flowmotion elements (README 11.1): file, name, body, length, loops, stop mode, parts.
const ELEMENTS: [(&str, &str, usize, i32, bool, u32, usize); 19] = [
    ("p_fa_ajc0", "P_FA_AJC_CAL0_0", 0, 35, false, 3, 6),
    ("p_fa_ajc0", "P_FA_AJC_SHO0_0", 1, 45, false, 0, 20),
    ("p_fa_akd0", "P_FA_AKD_DRL0_0", 0, 30, true, 3, 11),
    ("p_fa_akd0", "P_FA_AKD_CAL0_0", 1, 30, false, 3, 5),
    ("p_fa_akd0", "P_FA_AKD_SHO0_0", 2, 45, false, 3, 16),
    ("p_fa_ask0", "P_FA_ASK_MLC0_0", 0, 30, true, 3, 19),
    ("p_fa_com0", "P_FA_COM_WAL0_0", 11, 20, false, 0, 14),
    ("p_fa_com0", "P_FA_COM_BSS0_0", 12, 30, true, 3, 13),
    ("p_fa_com0", "P_FA_COM_LOC0_0", 9, 40, true, 3, 2),
    ("p_fa_com0", "P_FA_COM_LOC1_0", 10, 40, true, 3, 7),
    ("p_fa_com0", "P_FA_COM_LOC2_0", 19, 40, true, 3, 3),
    ("p_fa_com0", "P_FA_COM_MLC0_0", 16, 20, false, 3, 11),
    ("p_fa_com0", "P_FA_COM_FIR0_0", 17, 20, false, 3, 18),
    ("p_ac_com0", "P_AC_COM_MSL0_0", 0, 30, true, 3, 2),
    ("p_ac_com0", "P_AC_COM_MSS0_0", 1, 30, true, 3, 7),
    ("p_ac_com0", "P_AC_COM_RNG0_0", 6, 30, false, 0, 5),
    ("p_fa_bos_0", "P_FA_BOS_BSS0_0", 0, 30, true, 3, 6),
    ("p_fa_bos_0", "P_FA_BOS_LOC1_0", 1, 40, true, 3, 5),
    ("w_010_so", "W_010_SO_LOC0_0", 5, 6, true, 3, 1),
];

fn path_of(stem: &str) -> String {
    let rel = FILES.iter().find(|f| f.ends_with(&format!("/{stem}.fep"))).unwrap();
    format!("{ROOT}/{rel}")
}

/// The FEP, or `None` (with a message) when DDD's files are not here.
fn load(stem: &str) -> Option<Arc<Fep>> {
    let path = path_of(stem);
    let Ok(data) = std::fs::read(&path) else {
        eprintln!("skipped: {path} not found");
        return None;
    };
    Some(Arc::new(Fep::parse(data).unwrap_or_else(|e| panic!("{path}: {e}"))))
}

#[test]
fn seven_files_parse() {
    for rel in FILES {
        let stem = Path::new(rel).file_stem().unwrap().to_str().unwrap();
        let Some(f) = load(stem) else { return };
        assert_eq!(f.header.file_size as usize, f.data.len(), "{rel}");
        assert_eq!(f.bodies.len(), f.header.counts[1] as usize);
        // Every name resolves (4145/4145 game-wide).
        for n in &f.names {
            f.element(&n.name).unwrap_or_else(|e| panic!("{rel} {}: {e}", n.name));
        }
        // Local resource ids follow README 9.1.
        for (kind, list) in [(fep::ResKind::Tex, &f.tex), (fep::ResKind::Mdl, &f.mdl), (fep::ResKind::Anm, &f.anm), (fep::ResKind::Vtl, &f.vtl)] {
            for r in list.iter().filter(|r| r.id != 0 && !r.is_resident()) {
                assert_eq!(r.id, f.local_id(kind, r.index), "{rel} {kind:?} {}", r.index);
                let d = f.local_data(r).unwrap();
                let magic = match kind {
                    fep::ResKind::Tex => b"TEX",
                    fep::ResKind::Anm => b"ANM",
                    fep::ResKind::Mdl => b"MDL",
                    fep::ResKind::Vtl => b"VTL",
                };
                assert_eq!(&d[..3], magic, "{rel} {kind:?} {}", r.index);
            }
        }
    }
}

#[test]
fn resident_lookup() {
    let path = format!("{ROOT}/resident/bin/resident.frr");
    let Ok(data) = std::fs::read(&path) else {
        eprintln!("skipped: {path} not found");
        return;
    };
    let frr = Resident::parse(data).unwrap();
    assert_eq!(frr.tables.iter().map(|t| t.len()).collect::<Vec<_>>(), [32, 134, 136, 24]);
    // Every resident id the seven files name exists, with its magic.
    for rel in FILES {
        let stem = Path::new(rel).file_stem().unwrap().to_str().unwrap();
        let Some(f) = load(stem) else { return };
        for r in f.tex.iter().chain(&f.mdl).chain(&f.anm).chain(&f.vtl).filter(|r| r.is_resident()) {
            let d = fep::res::data(&f, Some(&frr), r).unwrap_or_else(|| panic!("{rel}: {:#x} missing", r.id));
            let magic = [b"TEX", b"ANM", b"MDL", b"VTL"][Resident::decode(r.id).unwrap().0 as usize];
            assert_eq!(&d[..3], magic, "{rel} {:#x}", r.id);
        }
    }
}

/// Part counts, lifetimes, slots, keys, loops and matrix types agree with tools/ddd_fep_dump.py.
#[test]
fn parts_agree_with_dumps() {
    for rel in FILES {
        let stem = Path::new(rel).file_stem().unwrap().to_str().unwrap();
        let Some(f) = load(stem) else { return };
        let Ok(dump) = std::fs::read_to_string(format!("{DUMPS}/{stem}.txt")) else {
            eprintln!("skipped: no dump for {stem}");
            return;
        };
        // (length, parts) per body; (life, slots, keys, loop start, loop end, matrix) per part.
        let num = |line: &str, before: &str, after: &str| -> i64 {
            let i = line.find(before).unwrap_or_else(|| panic!("{before:?} in {line:?}")) + before.len();
            let rest = &line[i..];
            let rest = rest.trim_start();
            let j = if after.is_empty() { rest.len() } else { rest.find(after).unwrap() };
            rest[..j].trim().parse().unwrap_or_else(|_| panic!("number in {line:?}"))
        };
        let mut bodies: Vec<(i64, i64, Vec<[i64; 6]>)> = Vec::new();
        let lines: Vec<&str> = dump.lines().collect();
        for (k, l) in lines.iter().enumerate() {
            if l.starts_with("  body ") && l.contains(" at 0x") {
                bodies.push((0, 0, Vec::new()));
            } else if l.contains("s32 length ") && l.starts_with("    +0x02") {
                bodies.last_mut().unwrap().0 = num(l, "s32 length", "frames");
            } else if l.contains(" spawn events  +0x12 u16 ") {
                bodies.last_mut().unwrap().1 = num(l, "+0x12 u16", "parts");
            } else if l.starts_with("    part ") && l.contains(" at 0x") {
                let (a, b, c) = (lines[k + 1], lines[k + 2], lines[k + 3]);
                bodies.last_mut().unwrap().2.push([
                    num(a, "life", "frames"),
                    num(b, "+0x08 slots", "+0x09"),
                    num(c, "key lists", ""),
                    num(c, "loop start", "+0x0e"),
                    num(c, "loop end", "+0x10"),
                    num(b, "+0x0b u8", "(matrix"),
                ]);
            }
        }
        assert_eq!(bodies.len(), f.bodies.len(), "{stem}");
        for (b, (body, (len, n, parts))) in f.bodies.iter().zip(&bodies).enumerate() {
            assert_eq!(body.length as i64, *len, "{stem} body {b}");
            assert_eq!(body.parts.len() as i64, *n, "{stem} body {b}");
            assert_eq!(body.parts.len(), parts.len(), "{stem} body {b}");
            for (k, (p, d)) in body.parts.iter().zip(parts).enumerate() {
                let p = p.as_ref().unwrap();
                let got = [p.lifetime as i64, p.slots.len() as i64, p.keys.len() as i64, p.loop_start as i64, p.loop_end as i64, p.matrix_type as i64];
                assert_eq!(&got, d, "{stem} body {b} part {k}");
            }
        }
    }
}

/// Run an element to its end: a looping one for three lengths, then a stop request. Returns
/// the most live parts seen and every part index seen alive.
fn run_to_end(f: &Arc<Fep>, name: &str, dt: f32) -> (usize, Vec<usize>) {
    let mut e = Instance::start(f.clone(), name, 0.0).unwrap_or_else(|err| panic!("{name}: {err}"));
    let (mut most, mut seen) = (0, vec![]);
    let mut steps = 0;
    let stop_at = (e.length() as f32 * 3.0 / dt) as usize;
    while !e.is_ended() {
        let live: Vec<_> = e.parts().collect();
        most = most.max(live.len());
        for p in &live {
            if !seen.contains(&p.index()) {
                seen.push(p.index());
            }
            for d in p.draws() {
                assert!(d.local_time.is_finite(), "{name}");
            }
            if let Some(m) = p.local() {
                assert!(m.is_finite(), "{name} part {} local {m:?}", p.index());
            }
        }
        if steps == stop_at {
            e.stop();
        }
        e.step(dt);
        steps += 1;
        assert!(steps < 20_000, "{name}: did not end (time {}, {} parts)", e.time(), e.part_count());
    }
    seen.sort();
    (most, seen)
}

#[test]
fn nineteen_elements_run_to_their_end() {
    for (stem, name, body, length, loops, stop, nparts) in ELEMENTS {
        let Some(f) = load(stem) else { return };
        let (_, _, b) = f.element(name).unwrap();
        assert_eq!(b, body, "{name}");
        let bd = &f.bodies[b];
        assert_eq!((bd.length, bd.flags & 2 != 0, bd.stop_mode, bd.parts.len()), (length, loops, stop, nparts), "{name}");
        for dt in [1.0, 0.5, 0.25] {
            let (most, seen) = run_to_end(&f, name, dt);
            assert!(most > 0, "{name}: no parts");
            eprintln!("{name} dt {dt}: up to {most} live parts; parts seen {seen:?}");
        }
    }
}

/// The dive's rings (parts 12..15) are children of part 16's Generateanchor (README 12).
#[test]
fn dive_rings_come_from_part_16() {
    let Some(f) = load("p_fa_ajc0") else { return };
    let (_, seen) = run_to_end(&f, "P_FA_AJC_SHO0_0", 0.5);
    for k in [12, 13, 14, 15, 16] {
        assert!(seen.contains(&k), "part {k} never lived: {seen:?}");
    }
}

/// README 8.1: p_fa_ajc0 element 0, part 2, Sclac (slot 2): 200 at frame 0, 341.04 at its
/// key at frame 9, 216.33 at frame 15 (the ac key rebase keeps it continuous); part 1, Colac
/// (slot 3): 160 at frame 0 -> 128 at its key at frame 18.
#[test]
fn ac_rebase_numbers() {
    let Some(f) = load("p_fa_ajc0") else { return };
    let mut e = Instance::start(f.clone(), "P_FA_AJC_CAL0_0", 0.0).unwrap();
    // Part 1 also comes from part 2's Generateanchorvertex, after the first update: a root copy
    // started now gives clean part times.
    let p1 = e.spawn_root(1, 0.0).unwrap();
    let mut scl = Vec::new();
    let mut col = Vec::new();
    for _ in 0..=18 {
        for p in e.parts() {
            if p.index() == 2 {
                scl.push((p.time(), p.output(2).unwrap().x));
            }
            if p.id() == p1 {
                col.push((p.time(), p.output(3).unwrap().x));
            }
        }
        e.step(1.0);
    }
    let at = |v: &[(f32, f32)], t: f32| v.iter().find(|(pt, _)| *pt == t).map(|x| x.1).unwrap_or_else(|| panic!("no frame {t}: {v:?}"));
    let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
    assert!(close(at(&scl, 0.0), 2.0), "{scl:?}");
    assert!(close(at(&scl, 9.0), 3.41042), "{scl:?}");
    assert!(close(at(&scl, 15.0), 2.16333), "{scl:?}");
    // Between keys it follows base + v t + a t^2 / 2 (percent x 0.01).
    let t: f32 = 4.0;
    assert!(close(at(&scl, 4.0), (200.0 + 34.0022 * t - 4.07353 * t * t / 2.0) * 0.01), "{scl:?}");
    // Part 1 was spawned after the first frame's update: its frame 0 is the next step.
    assert!(close(at(&col, 1.0), 160.0), "{col:?}");
    assert!(close(at(&col, 18.0), 128.0), "{col:?}");
    // Just before its key at 18 it is already there: continuous.
    assert!((at(&col, 17.0) - 128.0).abs() < 8.0, "{col:?}");
}

/// A stop request on a stop-mode-0 element fades over 30 frames, then ends it.
#[test]
fn stop_fades_out() {
    let Some(f) = load("p_fa_ajc0") else { return };
    let mut e = Instance::start(f, "P_FA_AJC_SHO0_0", 0.0).unwrap();
    for _ in 0..5 {
        e.step(1.0);
    }
    e.stop();
    e.step(1.0);
    assert!((e.fade() - (1.0 - 1.0 / 30.0)).abs() < 1e-6, "{}", e.fade());
    for _ in 0..28 {
        e.step(1.0);
    }
    assert!(!e.is_ended());
    e.step(1.0);
    assert!(e.is_ended());
}

#[test]
fn start_frame_and_errors() {
    let Some(f) = load("p_fa_ajc0") else { return };
    assert!(Instance::start(f.clone(), "P_FA_AJC_SHO0_0", 45.0).is_err());
    assert!(Instance::start(f.clone(), "NOPE", 0.0).is_err());
    let e = Instance::start(f, "P_FA_AJC_SHO0_0", 12.0).unwrap();
    assert_eq!(e.time(), 12.0);
    // Events up to frame 12 have fired: part 17 starts at frame 12 (README 12).
    assert!(e.parts().any(|p| p.index() == 17));
}

/// FecMegaanm_plus (draw-sprites.md 5): one part's sprites per part frame at dt 1, as (part
/// time, sprites) for the frames the part lives.
fn swarm(stem: &str, name: &str, part: usize) -> Option<Vec<(f32, Vec<fep::Sprite>)>> {
    let f = load(stem)?;
    let frr = Arc::new(Resident::parse(std::fs::read(format!("{ROOT}/resident/bin/resident.frr")).ok()?).unwrap());
    let opts = fep::Options { resident: Some(frr), ..Default::default() };
    let mut e = Instance::start_with(f, name, 0.0, opts).unwrap();
    let mut out = Vec::new();
    while !e.is_ended() {
        for p in e.parts().filter(|p| p.index() == part) {
            for d in p.draws().filter(|d| d.class == 153) {
                assert_eq!(d.blend_index(), Some(1), "{name} part {part}");
                assert!(d.mega_depth().is_some_and(|z| z < 0.0), "{name} part {part}");
                out.push((p.time(), d.sprites.clone()));
            }
        }
        e.step(1.0);
    }
    Some(out)
}

/// The ages a swarm must show, oldest first: emissions `(part frame k, n)` (with the
/// accumulator at 0, so s = n and the r-th is r / n old after its first update, 5.6) give at
/// part time t the ages `t - k + r / n`, r = n .. 1; those under `life` are drawn.
fn expected_ages(t: f32, emissions: &[(f32, u32)], life: f32) -> Vec<f32> {
    let mut v = Vec::new();
    for &(k, n) in emissions.iter().filter(|e| e.0 <= t) {
        for r in (1..=n).rev() {
            let a = t - k + r as f32 / n as f32;
            if a < life {
                v.push(a);
            }
        }
    }
    v
}

fn check_ages(what: &str, run: &[(f32, Vec<fep::Sprite>)], emissions: &[(f32, u32)], life: f32, n: usize) {
    assert!(run.iter().any(|(_, s)| !s.is_empty()), "{what}: no sprites");
    for (t, s) in run {
        let ages: Vec<f32> = s.iter().map(|s| s.age).collect();
        let want = expected_ages(*t, emissions, life);
        assert!(ages.len() == want.len() && ages.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-4), "{what} at {t}: {ages:?}, want {want:?}");
        assert!(s.len() <= n, "{what}: more than N");
    }
    assert!(run.last().unwrap().1.is_empty(), "{what}: sprites left at the end");
}

/// Each sprite through the run, followed by its age (+1 a frame).
fn tracks(run: &[(f32, Vec<fep::Sprite>)]) -> Vec<Vec<fep::Sprite>> {
    let mut done: Vec<Vec<fep::Sprite>> = Vec::new();
    let mut open: Vec<Vec<fep::Sprite>> = Vec::new();
    for (_, s) in run {
        let mut next = Vec::new();
        for sp in s {
            match open.iter().position(|tr| (tr.last().unwrap().age + 1.0 - sp.age).abs() < 1e-4) {
                Some(k) => {
                    let mut tr = open.swap_remove(k);
                    tr.push(*sp);
                    next.push(tr);
                }
                None => next.push(vec![*sp]),
            }
        }
        done.append(&mut open);
        open = next;
    }
    done.append(&mut open);
    done
}

/// The rings (shape 7, radius mode 1 / 0 with 0 %: radius exactly R): in the part's engine XY
/// plane at R, radial speed 0.15 acc -0.01, engine z velocity -0.05 acc +0.001 (5.6); fade-in
/// 0 (alpha x age under 1), fade-out 3 over life 16; flip mode 3; spin 70 a frame from one of
/// the blocks' angles `bases` (keys change it) +-180.
fn check_ring(what: &str, run: &[(f32, Vec<fep::Sprite>)], r: f32, bases: &[f32]) {
    let mut flips = [0; 4];
    for tr in tracks(run) {
        let a0 = tr[0].angle - 70.0 * tr[0].age;
        assert!(bases.iter().any(|b| (-180.0..180.0).contains(&(a0 - b))), "{what}: angle {a0}");
        let d0 = tr[0].position.truncate().normalize();
        for sp in &tr {
            let (a, h) = (sp.age, sp.age * sp.age * 0.5);
            let q = sp.position;
            assert!(!sp.world, "{what}");
            let want_r = r + 0.15 * a - 0.01 * h;
            assert!((q.truncate().length() - want_r).abs() < 1e-4, "{what}: radius {} at age {a}, want {want_r}", q.truncate().length());
            assert!((q.z - (-0.05 * a + 0.001 * h)).abs() < 1e-5, "{what}: z {} at age {a}", q.z);
            assert!((q.truncate().normalize() - d0).length() < 1e-4, "{what}: turned");
            let fade = if a < 1.0 {
                a
            } else if a > 13.0 {
                1.0 - (a - 13.0) / 3.0
            } else {
                1.0
            };
            assert!((sp.colour.w - 128.0 * fade).abs() < 1e-3, "{what}: alpha {} at age {a}", sp.colour.w);
            assert!((sp.angle - a0 - 70.0 * a).abs() < 1e-3, "{what}: angle");
            assert_eq!(sp.scale.z, 100.0);
            flips[sp.flip as usize] += 1;
        }
    }
    assert!(flips.iter().filter(|&&n| n > 0).count() > 1, "{what}: flips {flips:?}");
}

/// Dive SHO0 part 11 (5.6): box (0.2, 1, 0.2), no radial speed, engine y acceleration 0.008
/// +- 0.002; N 7, life 13; rate 1 (one a frame), interval 2 from frame 5 (frames 5, 8), rate 0
/// from 11.
#[test]
fn dive_sparks() {
    let Some(run) = swarm("p_fa_ajc0", "P_FA_AJC_SHO0_0", 11) else { return };
    let em: Vec<(f32, u32)> = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 8.0].iter().map(|&k| (k, 1)).collect();
    check_ages("dive 11", &run, &em, 13.0, 7);
    let tr = tracks(&run);
    assert_eq!(tr.len(), 7);
    for t in &tr {
        let (p0, a0) = (t[0].position, t[0].age);
        assert!(p0.x.abs() <= 0.2 && p0.z.abs() <= 0.2 && p0.y.abs() <= 1.0 + 0.01 * a0 * a0, "{p0}");
        for sp in &t[1..] {
            let q = sp.position;
            assert!(q.x == p0.x && q.z == p0.z, "dive 11: moved sideways");
            let acc = (q.y - p0.y) / ((sp.age * sp.age - a0 * a0) * 0.5);
            assert!((0.006 - 1e-4..0.010 + 1e-4).contains(&acc), "dive 11: y acc {acc}");
            // Colour (140, 120, 80) +- 10, green -4 a frame, alpha 128.
            assert_eq!(sp.colour.w, 128.0);
            assert!(((sp.colour.y + 4.0 * sp.age) - (t[0].colour.y + 4.0 * a0)).abs() < 1e-3);
            // Scale 28 +- 10, the same random on x and y (mode 0).
            assert!(sp.scale.x == sp.scale.y && (18.0..38.0).contains(&sp.scale.x));
        }
    }
}

/// Dive SHO0 part 17 (5.6): the ring of radius 4, rate 3 (frames 0-3) then 1 with interval 2
/// (frames 4, 7, 10), rate 0 from 12; N 15, life 16.
#[test]
fn dive_ring_sparks() {
    let Some(run) = swarm("p_fa_ajc0", "P_FA_AJC_SHO0_0", 17) else { return };
    let em = [(0.0, 3), (1.0, 3), (2.0, 3), (3.0, 3), (4.0, 1), (7.0, 1), (10.0, 1)];
    check_ages("dive 17", &run, &em, 16.0, 15);
    check_ring("dive 17", &run, 4.0, &[0.0, 280.0, 840.0]);
}

/// Slam SHO0 part 14 (5.6): the ring of radius 1.3, rate 2 (frames 0-4) then 1 with interval
/// 2 (frames 5, 8, .., 17), rate 0 from 20; N 15, life 16.
#[test]
fn slam_ring_sparks() {
    let Some(run) = swarm("p_fa_akd0", "P_FA_AKD_SHO0_0", 14) else { return };
    let em = [(0.0, 2), (1.0, 2), (2.0, 2), (3.0, 2), (4.0, 2), (5.0, 1), (8.0, 1), (11.0, 1), (14.0, 1), (17.0, 1)];
    check_ages("slam 14", &run, &em, 16.0, 15);
    check_ring("slam 14", &run, 1.3, &[0.0, 210.0, 350.0]);
}

/// Slam SHO0 part 15: box, rate 1 with interval 1 (every other frame) until frame 11, N 5,
/// life 20: frame 10's sprite finds the buffer full and is lost (5.3).
#[test]
fn slam_sparks_full_buffer() {
    let Some(run) = swarm("p_fa_akd0", "P_FA_AKD_SHO0_0", 15) else { return };
    let em = [(0.0, 1), (2.0, 1), (4.0, 1), (6.0, 1), (8.0, 1)];
    check_ages("slam 15", &run, &em, 20.0, 5);
    for (_, s) in &run {
        for sp in s {
            assert_eq!(sp.colour.w, 140.0);
        }
    }
}

/// resident.frr, or `None` when DDD's files are not here.
fn frr() -> Option<Arc<Resident>> {
    Some(Arc::new(Resident::parse(std::fs::read(format!("{ROOT}/resident/bin/resident.frr")).ok()?).unwrap()))
}

fn start(stem: &str, name: &str, matrix: glam::Mat4) -> Option<Instance> {
    let f = load(stem)?;
    let opts = fep::Options { resident: Some(frr()?), matrix, ..Default::default() };
    Some(Instance::start_with(f, name, 0.0, opts).unwrap())
}

/// generators.md 5.5: the dive's part 19 (Generateanchorvertex, MDL 9, VTL list 0 = [1, 2, 3, 4,
/// 0], count 5, in order) spawns part 18 (the trail) on the five pentagon corners at once,
/// placed by part 19's local matrix (child type 0).
#[test]
fn dive_trail_parts_on_model_vertices() {
    let Some(mut e) = start("p_fa_ajc0", "P_FA_AJC_SHO0_0", glam::Mat4::IDENTITY) else { return };
    let corners = [(-0.476, -0.155), (-0.294, 0.405), (0.294, 0.405), (0.476, -0.155), (0.0, -0.5)];
    for _ in 0..45 {
        let found = e.parts().find(|p| p.index() == 19).map(|g| (g.id(), g.local().unwrap().inverse()));
        if let Some((gid, inv)) = found {
            let kids: Vec<_> = e.parts().filter(|p| p.index() == 18 && p.parent() == Some(gid)).collect();
            assert_eq!(kids.len(), 5, "one burst of five");
            for (k, (x, z)) in kids.iter().zip(corners) {
                let v = inv.transform_point3(k.local().unwrap().w_axis.truncate());
                assert!((v - glam::Vec3::new(x, 0.0, z)).length() < 2e-3, "corner {v:?} vs {x} {z}");
                assert_eq!(k.time(), 0.0);
            }
            drop(kids);
            // The burst is not repeated (part 19 has a frame-0 block only); part 18 lives 19 frames.
            for _ in 0..18 {
                e.step(1.0);
                assert_eq!(e.parts().filter(|p| p.index() == 18).count(), 5);
            }
            return;
        }
        e.step(1.0);
    }
    panic!("part 19 never lived");
}

/// tails.md 1.4 / 1.7 on the dive's trail (part 18; Locus slot 6: spacing 0.12, N 17, space 0;
/// Shapetail slot 7: size 5% -> 30%, colour (160, 110, 120, 128) -> (64, 120, 32, 0) x Colac).
#[test]
fn dive_trail_records_points() {
    let Some(mut e) = start("p_fa_ajc0", "P_FA_AJC_SHO0_0", glam::Mat4::IDENTITY) else { return };
    let mut id = None;
    let (mut last_v, mut frames) = (0, 0);
    while !e.is_ended() {
        if id.is_none() {
            id = e.parts().find(|p| p.index() == 18).map(|p| p.id());
        }
        let Some(p) = id.and_then(|id| e.parts().find(|p| p.id() == id)) else {
            if id.is_some() {
                break; // it died
            }
            e.step(1.0);
            continue;
        };
        frames += 1;
        let (pts, v) = p.locus(6).unwrap();
        assert_eq!(pts.len(), 17);
        assert!(v >= last_v && v <= 17, "reached {v} after {last_v}");
        last_v = v;
        // The newest point is the part's origin (route 0 through its local matrix).
        assert!((pts[0] - p.local().unwrap().w_axis.truncate()).length() < 1e-5);
        // A chord is at most the spacing (arc length 0.12 along the recorded path).
        for w in pts[..v].windows(2) {
            assert!((w[1] - w[0]).length() <= 0.12 + 1e-4, "gap {}", (w[1] - w[0]).length());
        }
        let d = p.draws().find(|d| d.class == 80).unwrap();
        assert_eq!(d.resource, Some(7));
        if let Some(t) = &d.tail {
            assert_eq!((t.reached, t.points.len(), t.world, t.facing, t.blend), (v, v, false, true, 4));
            let c = p.output(3).unwrap();
            let lim = |v: glam::Vec4| v.clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0));
            let head = lim(c * glam::Vec4::new(160.0, 110.0, 120.0, 128.0) / 128.0);
            assert!((t.points[0].colour - head).length() < 1e-3);
            let k = (v - 1) as f32 / v as f32;
            let tail = c * glam::Vec4::new(64.0, 120.0, 32.0, 0.0) / 128.0;
            let unclamped = c * glam::Vec4::new(160.0, 110.0, 120.0, 128.0) / 128.0;
            if lim(unclamped) == unclamped && lim(tail) == tail {
                assert!((t.points[v - 1].colour - (unclamped + (tail - unclamped) * k)).length() < 1e-2);
            }
            // No random size: the size factor only (rotations random, 0..180).
            assert!((t.points[v - 1].scale - (0.05 + 0.25 * k)).abs() < 1e-5);
            assert!(t.points.iter().all(|q| (0.0..180.0).contains(&q.angle)));
            // ANM 19 keys x 2 frames at speed 1; the clock had the spawn frame's dt (1) already.
            assert_eq!(t.anm_frame as f32, ((p.time() + 1.0) / 2.0).floor().min(18.0), "time {}", p.time());
        }
        e.step(1.0);
    }
    assert!(frames > 10, "part 18 lived {frames} frames");
    assert_eq!(last_v, 17, "the trail never grew to full length");
}

/// The rail trail (COM LOC0, space 1 = world): with the instance matrix moving 0.2 a frame along
/// x, the points lie behind the part on that line at the spacing (0.24 / 0.10), newest first.
#[test]
fn rail_trail_follows_the_matrix() {
    let Some(mut e) = start("p_fa_com0", "P_FA_COM_LOC0_0", glam::Mat4::IDENTITY) else { return };
    for f in 1..30 {
        e.set_matrix(glam::Mat4::from_translation(glam::Vec3::new(0.2 * f as f32, 0.0, 0.0)));
        e.step(1.0);
    }
    let x = 0.2 * 29.0;
    for (part, spacing) in [(0usize, 0.24f32), (1, 0.10)] {
        let p = e.parts().find(|p| p.index() == part).unwrap();
        let (pts, v) = p.locus(6).unwrap();
        assert_eq!(v, 17, "part {part}");
        let origin = p.frame().transform_point3(p.local().unwrap().w_axis.truncate());
        assert!((pts[0] - origin).length() < 1e-4, "part {part}: {:?} vs {origin:?}", pts[0]);
        for (k, q) in pts.iter().enumerate() {
            let want = origin - glam::Vec3::new(spacing * k as f32, 0.0, 0.0);
            assert!((*q - want).length() < 1e-3, "part {part} point {k}: {q:?} vs {want:?} (x {x})");
        }
        let t = p.draws().find(|d| d.class == 80).and_then(|d| d.tail).unwrap();
        assert_eq!((t.world, t.facing, t.blend, t.reached), (true, true, 1, 17));
        assert!(t.orient.is_some());
    }
}

/// generators.md 7.3: COM BSS0 part 2 (fade 2, B alpha 0.5): the factor after each update is
/// 0.75, 0.5, 0.75, 1.0, ... and is applied to the Colac in place; part 12 (fade 1) alternates B,
/// A from the first update.
#[test]
fn colorblink_changes_the_colour() {
    let Some(mut e) = start("p_fa_com0", "P_FA_COM_BSS0_0", glam::Mat4::IDENTITY) else { return };
    let (mut f2, mut f12) = (Vec::new(), Vec::new());
    for t in 0..8 {
        for p in e.parts().filter(|p| p.index() == 2 || p.index() == 12) {
            let (blink, colac, copy) = (p.output(4).unwrap(), p.output(3).unwrap(), p.output(5).unwrap());
            if p.index() == 2 {
                if t == 0 {
                    // Colorblink's own copy is the Colac before the factor.
                    assert_eq!(colac, (copy * blink).clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0)));
                }
                f2.push(blink);
            }
            if p.index() == 12 {
                f12.push(blink);
            }
        }
        e.step(1.0);
    }
    let a: Vec<f32> = f2.iter().map(|v| v.w).collect();
    assert_eq!(&a[..6], &[0.75, 0.5, 0.75, 1.0, 0.75, 0.5], "{f2:?}");
    assert!(f2.iter().all(|v| v.truncate() == glam::Vec3::ONE));
    let (b, a) = (glam::Vec4::new(1.0, 0.4, 0.4, 0.3), glam::Vec4::new(1.0, 0.9, 0.7, 1.0));
    for (k, v) in f12.iter().enumerate() {
        let want = if k % 2 == 0 { b } else { a };
        assert!((*v - want).length() < 1e-5, "update {k}: {v:?}");
    }
    assert!(f12.len() >= 4);
}

/// The ribbons of an instance: (part index, its Colac / Col input, the draw).
fn ribbons(e: &Instance) -> Vec<(usize, glam::Vec4, fep::RibbonDraw)> {
    e.parts().flat_map(|p| p.draws().filter_map(|d| Some((p.index(), d.colour?, d.ribbon?))).collect::<Vec<_>>()).collect()
}

/// The line draws of an instance: (part index, its colour input, the draw).
fn line_draws(e: &Instance) -> Vec<(usize, glam::Vec4, fep::LineDraw)> {
    e.parts().flat_map(|p| p.draws().filter_map(|d| Some((p.index(), d.colour?, d.lines?))).collect::<Vec<_>>()).collect()
}

/// tails.md 2 and 6: the Keyblade trail (w_010_so LOC0). N = 6 samples a update, B-spline S 32
/// -> 29 points, 58 vertices; edges A (0, 0, -0.735) and B (0, 0, 0.0189) through Scl (1, 1,
/// 0.7); U 0 / 0.5 across, V from 1 down by 1/29; head colour = the Colac, tail 0, head fade
/// k = 2 (alpha x0, x0.5). With the matrix moving 0.1 along x each update, the strip's ends are
/// the newest and the 6th-newest samples.
#[test]
fn keyblade_ribbon() {
    let at = |u: usize| glam::Mat4::from_translation(glam::Vec3::new(0.1 * u as f32, 0.0, 0.0));
    let Some(mut e) = start("w_010_so", "W_010_SO_LOC0_0", at(0)) else { return };
    let (a, b) = (glam::Vec3::new(0.0, 0.0, -0.735), glam::Vec3::new(0.0, 0.0, 0.0189));
    for u in 0..12usize {
        let r = ribbons(&e);
        assert_eq!(r.len(), 1, "update {u}");
        let (part, colac, d) = &r[0];
        assert_eq!(*part, 0);
        let v = &d.vertices;
        assert_eq!(v.len(), 58);
        assert_eq!((d.texture, d.blend, d.sampler), (Some(8), 1, true));
        assert!((d.depth.unwrap() - glam::Vec3::new(0.0, 0.0, -0.1)).length() < 1e-6);
        // Newest sample: the part's points now; oldest: 5 updates back (all at the start before).
        let (newest, oldest) = (0.1 * u as f32, 0.1 * u.saturating_sub(5) as f32);
        let x = glam::Vec3::X;
        assert!((v[0].position - (a + x * newest)).length() < 1e-5, "update {u}: {:?}", v[0].position);
        assert!((v[1].position - (b + x * newest)).length() < 1e-5);
        assert!((v[56].position - (a + x * oldest)).length() < 1e-4, "update {u}: {:?}", v[56].position);
        assert!((v[57].position - (b + x * oldest)).length() < 1e-4);
        for k in 0..29 {
            assert!(v[2 * k].position.x <= newest + 1e-5 && v[2 * k].position.x >= oldest - 1e-5);
            assert_eq!((v[2 * k].uv.x, v[2 * k + 1].uv.x), (0.0, 0.5));
            assert!((v[2 * k].uv.y - (1.0 - k as f32 / 29.0)).abs() < 1e-5, "v {k}: {}", v[2 * k].uv.y);
            assert_eq!(v[2 * k].colour, v[2 * k + 1].colour);
            let c = colac.clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0)) * (1.0 - k as f32 / 29.0);
            let fade = [0.0, 0.5, 1.0][k.min(2)];
            let want = [c.x as u8, c.y as u8, c.z as u8, (c.w * fade) as u8];
            for j in 0..4 {
                assert!((v[2 * k].colour[j] as i32 - want[j] as i32).abs() <= 1, "update {u} point {k}: {:?} vs {want:?}", v[2 * k].colour);
            }
        }
        e.set_matrix(at(u + 1));
        e.step(0.5);
    }
}

/// tails.md 2 on AJC / AKD CAL0 (both parts): N 16, S 64 -> 61 points, 122 vertices; U flipped
/// (A edge 1, B edge 0); V scrolls 5 texels a frame (2.5 an update at dt 0.5) over 64: the head's
/// V goes up by 2.5 / 64 (mod 1) each update; head colour = (160, 140|132, 78, 128) x Col / 128,
/// no head fade; the wave keeps every sample near the part (amplitude 0.03 a update).
#[test]
fn cal0_ribbons() {
    for (stem, name, g) in [("p_fa_ajc0", "P_FA_AJC_CAL0_0", 140.0), ("p_fa_akd0", "P_FA_AKD_CAL0_0", 132.0)] {
        let Some(mut e) = start(stem, name, glam::Mat4::IDENTITY) else { return };
        let mut last: Vec<(usize, f32)> = Vec::new();
        let mut seen = 0;
        for _ in 0..40 {
            let r = ribbons(&e);
            let mut now = Vec::new();
            // Both parts' instances, in instance order (two share a part index).
            for (n, (_, col, d)) in r.iter().enumerate() {
                let v = &d.vertices;
                assert_eq!(v.len(), 122, "{name}");
                assert_eq!((d.texture.is_some(), d.blend, d.depth), (true, 1, None));
                assert_eq!((v[0].uv.x, v[1].uv.x), (1.0, 0.0));
                let head = (glam::Vec4::new(160.0, g, 78.0, 128.0) * *col / 128.0).clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0));
                let want = [head.x as u8, head.y as u8, head.z as u8, head.w as u8];
                assert_eq!(v[0].colour, want, "{name}: {col:?}");
                for k in 1..61 {
                    assert!((v[2 * k].uv.y - (v[0].uv.y - k as f32 / 61.0)).abs() < 1e-4);
                    assert!((v[2 * k].position - v[2 * k - 2].position).length() < 1.0, "{name}: the ribbon tears");
                }
                if let Some((_, was)) = last.iter().find(|(p, _)| *p == n).filter(|_| last.len() == r.len()) {
                    let dv = (v[0].uv.y - was - 2.5 / 64.0).rem_euclid(1.0);
                    assert!(dv < 1e-4 || dv > 1.0 - 1e-4, "{name}: V {was} -> {}", v[0].uv.y);
                }
                now.push((n, v[0].uv.y));
                seen += 1;
            }
            last = now;
            e.step(0.5);
            if e.is_ended() {
                break;
            }
        }
        assert!(seen > 40, "{name}: {seen} ribbon draws");
    }
}

/// tails.md 4 on ASK MLC0 part 15: one particle every 2 updates at dt 0.5 (rate 1, interval 0),
/// life 18 updates; 3-point strips on a circle about local Y (radius <= 10), each point one
/// update further back along the spin (10 +- 3 degrees); colour (128, 96, 64, 128) -> (128, 64,
/// 64, 0) in thirds, RGB 255 = 1 (x Colac / 255), alpha x the fade; matrix = local x diag(0.18,
/// 0.01, 0.18).
#[test]
fn ask_ellipse_lines() {
    let Some(mut e) = start("p_fa_ask0", "P_FA_ASK_MLC0_0", glam::Mat4::IDENTITY) else { return };
    let mut most = 0;
    for u in 0..60 {
        for (part, col, l) in line_draws(&e) {
            assert_eq!((part, l.facing, l.blend), (15, 0, 1));
            let local = e.parts().find(|p| p.index() == 15).unwrap().local().unwrap();
            assert!((l.matrix - local * glam::Mat4::from_diagonal(glam::Vec4::new(0.18, 0.01, 0.18, 1.0))).abs_diff_eq(glam::Mat4::ZERO, 1e-6));
            assert!(l.strips.len() <= 9, "update {u}: {} lines", l.strips.len());
            most = most.max(l.strips.len());
            let c = col.clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0)) / 255.0;
            for s in &l.strips {
                assert_eq!(s.len(), 3);
                let r = glam::Vec2::new(s[0].position.x, s[0].position.z).length();
                assert!(r <= 10.0);
                for (k, q) in s.iter().enumerate() {
                    assert!((q.position.y - s[0].position.y).abs() < 1e-4);
                    assert!((glam::Vec2::new(q.position.x, q.position.z).length() - r).abs() < 1e-3);
                    let h = glam::Vec3::new(128.0, 96.0 - 32.0 * k as f32 / 3.0, 64.0) / 128.0;
                    let rgb = (h * c.truncate()).clamp(glam::Vec3::ZERO, glam::Vec3::ONE) * 255.0;
                    assert_eq!(&q.colour[..3], &[rgb.x as u8, rgb.y as u8, rgb.z as u8]);
                    assert!(q.colour[3] as f32 <= (1.0 - k as f32 / 3.0) * c.w * 255.0 + 1.0);
                }
                // Spin between neighbours: 7..13 degrees.
                let ang = |q: &fep::LineVertex| q.position.z.atan2(q.position.x).to_degrees();
                if r > 0.01 {
                    let d = (ang(&s[1]) - ang(&s[0]) + 540.0).rem_euclid(360.0) - 180.0;
                    assert!((7.0 - 1e-3..=13.0 + 1e-3).contains(&d.abs()), "spin {d}");
                }
            }
        }
        e.step(0.5);
    }
    assert!(most >= 8, "at most {most} lines");
}

/// tails.md 5 on COM BSS0 part 4: speed 0.2, no acceleration or gravity, n 2: each line is one
/// segment of length 0.2 inside the cone about +Y (half-angle 40 degrees per axis); colours
/// (128, 96, 32, 128) and halfway to (128, 32, 32, 0); one emission per 26 updates per part
/// (interval 12 at dt 0.5), lives 8..12 updates.
#[test]
fn bss_radial_lines() {
    let Some(mut e) = start("p_fa_com0", "P_FA_COM_BSS0_0", glam::Mat4::IDENTITY) else { return };
    let mut lines = 0;
    for _ in 0..60 {
        for (part, col, l) in line_draws(&e) {
            assert_eq!((part, l.facing, l.blend), (4, 0, 1));
            assert!(l.strips.len() <= 1);
            let c = col.clamp(glam::Vec4::ZERO, glam::Vec4::splat(255.0)) / 255.0;
            for s in &l.strips {
                lines += 1;
                assert_eq!(s.len(), 2);
                let d = s[0].position - s[1].position;
                assert!((d.length() - 0.2).abs() < 1e-4, "{d:?}");
                let n = d.normalize();
                let cos40 = 40f32.to_radians().cos();
                assert!(n.y >= cos40 * cos40 - 1e-4, "{n:?}");
                for (k, h) in [(0, glam::Vec3::new(1.0, 0.75, 0.25)), (1, glam::Vec3::new(1.0, 0.5, 0.25))] {
                    let rgb = (h * c.truncate()).clamp(glam::Vec3::ZERO, glam::Vec3::ONE) * 255.0;
                    assert_eq!(&s[k].colour[..3], &[rgb.x as u8, rgb.y as u8, rgb.z as u8]);
                }
                assert!(s[1].colour[3] <= s[0].colour[3] / 2 + 1);
            }
        }
        e.step(0.5);
    }
    assert!(lines > 10, "{lines} lines drawn");
}

/// aura.md 5: COM FSH0 part 10's FecFlash. Init and key 0 alike: `+0x19` = 0 (the whole virtual
/// screen, 0, 0, 400, 240, though the block's rect is 512 x 448), v0 128 and rate 0 (value stays
/// 128), blend selector 1; colour = its Colac: (128, 64, 128) with alpha `8 + 59.72 t - 31.72 t^2
/// / 2` up to its key at frame 2 (64); part 10 lives 7 frames, so the flash is over by frame 7.
#[test]
fn fsh_flash() {
    let Some(mut e) = start("p_fa_com0", "P_FA_COM_FSH0_0", glam::Mat4::IDENTITY) else { return };
    let mut seen = Vec::new();
    for _ in 0..40 {
        for p in e.parts() {
            for d in p.draws() {
                let Some(f) = d.flash else { continue };
                assert_eq!(p.index(), 10);
                assert_eq!(d.status, fep::ClassStatus::DrawData);
                assert_eq!((f.rect, f.full_screen, f.blend, f.value), ([0, 0, 400, 240], true, 1, 128.0));
                assert_eq!(f.colour.truncate(), glam::Vec3::new(128.0, 64.0, 128.0));
                seen.push((p.time(), f.colour.w));
            }
        }
        e.step(0.5);
    }
    assert!(seen.len() >= 10, "{seen:?}");
    for &(t, a) in &seen {
        assert!(t < 7.0, "drawn at {t}");
        if t <= 2.0 {
            let want = 8.0 + 59.72 * t - 31.72 * t * t / 2.0;
            assert!((a - want).abs() < 0.05, "alpha {a} at {t}, want {want}");
        }
    }
    assert!(seen.iter().any(|&(t, a)| t == 2.0 && (a - 64.0).abs() < 0.05), "{seen:?}");
}

/// aura.md 7: the aura's sprites (ORA0 x3, ORA1, FOR0 parts 0-1, the Keyblade's ORA0 x7) are
/// one ANM 0 quad (size 62, vertices +-0.5) at Scl 5..9.5 %, and their depth push (`+0x08`, on,
/// view space) is stored **+0.3**: the opposite sign of every other Flowmotion sprite (-0.5..-0.8,
/// e.g. the dive CAL0's -0.8), i.e. depth-tested behind, not in front.
#[test]
fn aura_sprites_push_away() {
    let f32_at = |d: &[u8], o: usize| f32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let pushes = |stem: &str, name: &str| -> Option<Vec<(usize, f32, f32)>> {
        let mut e = start(stem, name, glam::Mat4::IDENTITY)?;
        let mut out = Vec::new();
        for _ in 0..10 {
            e.step(0.5);
        }
        for p in e.parts() {
            for d in p.draws().filter(|d| d.class == 152 && d.params.get(0x1d) == Some(&1)) {
                let scl = p.local().unwrap().x_axis.truncate().length();
                out.push((p.index(), f32_at(d.params, 8), scl));
            }
        }
        Some(out)
    };
    let Some(ora0) = pushes("p_fa_com0", "P_FA_COM_ORA0_0") else { return };
    assert_eq!(ora0.len(), 3);
    for (want, (_, z, scl)) in [0.06, 0.055, 0.05].iter().zip(&ora0) {
        assert_eq!(*z, 0.3);
        assert!((scl - want).abs() < 1e-6, "{scl}");
    }
    let ora1 = pushes("p_fa_com0", "P_FA_COM_ORA1_0").unwrap();
    assert_eq!(ora1.iter().map(|t| t.1).collect::<Vec<_>>(), [0.3]);
    let blade = pushes("w_010_so", "W_010_SO_ORA0_0").unwrap();
    assert!(blade.len() == 7 && blade.iter().all(|t| t.1 == 0.3), "{blade:?}");
    let cal0 = pushes("p_fa_ajc0", "P_FA_AJC_CAL0_0").unwrap();
    assert!(!cal0.is_empty() && cal0.iter().all(|t| t.1 == -0.8), "{cal0:?}");
}

#[test]
fn bad_files_are_errors() {
    assert!(Fep::parse(Vec::new()).is_err());
    assert!(Fep::parse(b"FEP\0\x01\x00\x09\x00".to_vec()).is_err());
    let Some(f) = load("w_010_so") else { return };
    // Every truncation of a real file is an error, never a panic.
    for n in (0..f.data.len()).step_by(97) {
        let _ = Fep::parse(f.data[..n].to_vec());
    }
    // A changed signature record is refused.
    let mut d = f.data.clone();
    d[0xf8 + 8 * 3 + 4] ^= 1;
    assert!(matches!(Fep::parse(d), Err(fep::Error::Signature(3))));
}

/// Every FEP of the game parses (667 files). `cargo test -- --ignored`.
#[test]
#[ignore]
fn all_667_parse() {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "fep") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(Path::new(ROOT), &mut files);
    if files.is_empty() {
        eprintln!("skipped: {ROOT} not found");
        return;
    }
    let mut elements = 0;
    for p in &files {
        let f = Fep::parse(std::fs::read(p).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        for n in &f.names {
            f.element(&n.name).unwrap_or_else(|e| panic!("{} {}: {e}", p.display(), n.name));
            elements += 1;
        }
    }
    eprintln!("{} files, {elements} elements", files.len());
    assert_eq!(files.len(), 667);
    assert_eq!(elements, 4145);
}
