# fep: DDD's effect interpreter

A Rust library that reads Dream Drop Distance's effect files (`effect/**/*.fep`) and plays
their elements as DDD's exe does (Steam KH 2.8; every address below is an RVA from image base
0x140000000). The format is in `../docs/fep/README.md` ("the spec"); this crate follows it,
and reads the exe where the spec is thin. It does not draw: it gives a drawer each live part's
matrices, colour and draw commands.

Package `fep`, edition 2021, MIT OR Apache-2.0, depends on `glam` 0.29 only. Build and test in
this folder (`cargo test`; `cargo test --release -- --ignored` parses all 667 FEPs).

## Use

```rust
let fep = Arc::new(fep::Fep::parse(std::fs::read(path)?)?);
let frr = Arc::new(fep::Resident::parse(std::fs::read(frr_path)?)?); // for resident ANMs
let opts = fep::Options { resident: Some(frr), ..Default::default() };
let mut e = fep::Instance::start_with(fep, "P_FA_AJC_SHO0_0", 0.0, opts)?;
loop {
    e.set_matrix(attachment);              // DDD's instance matrix (+0x60), see below
    for p in e.parts() {                   // live parts
        let (index, local, frame, colour, scale) = (p.index(), p.local(), p.frame(), p.colour(), p.scale());
        for d in p.draws() {               // the part's draw list
            let _ = (d.class, d.params, d.resource, d.local_time, d.colour, d.anm_frame());
            for s in &d.sprites {          // FecMegaanm_plus: its live sprites, oldest first
                let _ = (s.anm_frame, s.flip, s.position, s.world, s.scale, s.angle, s.colour);
            }
            if let Some(t) = &d.tail {     // FecShapetail: one ANM sprite per trail point, head first
                for q in &t.points {
                    let _ = (t.anm_frame, t.world, t.facing, t.blend, q.position, q.scale, q.angle, q.colour);
                }
            }
            if let Some(r) = &d.ribbon {   // FecBindlocus_polygon: a triangle strip A0 B0 A1 B1 ...
                for v in &r.vertices {
                    let _ = (r.texture, r.blend, r.depth, v.position, v.uv, v.colour);
                }
            }
            if let Some(l) = &d.lines {    // line particles: one strip per particle, newest first
                for s in &l.strips {
                    let _ = (l.matrix, l.facing, l.blend, s[0].position, s[0].colour);
                }
            }
            if let Some(f) = &d.flash {    // FecFlash: a rectangle of the 400 x 240 virtual screen
                let _ = (f.rect, f.full_screen, f.colour, f.blend);
            }
        }
    }
    if stop_wanted { e.stop(); }
    e.step(dt);                            // dt in FEP frames
    if e.is_ended() { break; }
}
```

| item | what |
|---|---|
| `Fep::parse(bytes)` | the whole file; `Err`, never a panic, on bad data |
| `Fep::element(name)` / `element_by_id(id)` | group, element, body (`0x5abb70`, `0x5acb40`) |
| `Fep::resource(kind, i)`, `resource_by_id(id)`, `local_data(r)`, `local_id(kind, i)` | sections 2..5 |
| `Resident::parse`, `get(id)`, `by_index(kind, k)`, `res::data(fep, frr, r)` | resident.frr and any resource's bytes |
| `Instance::start(fep, name, start_frame)` / `start_with(.., Options)` | create and start an element; `Options`: `rng`, `resident`, `matrix`, `world_unit` (deviation 15) |
| `Instance::step(dt)` | one frame |
| `Instance::stop()` | the stop request |
| `Instance::set_matrix(m)` | the instance matrix the parts' frames come from |
| `Instance::parts()` | live parts: `index`, `id`, `parent`, `time`, `lifetime`, `local`, `world`, `frame`, `matrix_type`, `lmatrix_flags`, `colour`, `scale`, `position`, `output(slot)`, `locus(slot)` (a Locus slot's N points, newest first, and how many are reached), `draws()` |
| `DrawCmd` | `slot`, `class`, `name`, `status`, `params` (bytes), `params_offset`, `resource` (Shapetail: its ANM `+0x2c`), `local_time`, `work`, `colour`, `sprites`, `tail`, `ribbon`, `lines`, `flash`, `anm_frame()`, `mdlts_value()`, `blend_index()`, `tail_blend()`, `mega_depth()` |
| `RibbonDraw` / `RibbonVertex` | FecBindlocus_polygon's strip as its update rebuilt it (tails.md 2.5, 2.7): `vertices` (`A0 B0 A1 B1 ...`, 2 per point, point 0 the newest sample; `position` in the frame's space = DDD's world, `uv` (1.0 = the texture's size), `colour` bytes with the head fade, 128 = 1.0 for the textured preset), `texture` (section-2 index, `None` untextured), `blend` (`+0x51`, an index for `0x7ed8e0`), `sampler`, `depth` (`+0x20` when `+0x55`) |
| `LineDraw` / `LineVertex` | FecEllipse_line_particle / FecRadialg_line_particle (tails.md 4.4, 5.4): `strips` (one per live particle, slot order, newest point first; `colour` bytes, RGB 255 = 1.0, alpha 128 = 1.0: untextured preset B = 0x201, render-state.md 3), `matrix` (Ellipse `diag(axes) x local`, RadialG `local`; the drawer applies `0x5b2190` by `facing` = Lmatrix byte 1, and the frame), `blend` (an index for `0x7ed8e0`) |
| `FlashDraw`, `SCREEN` | FecFlash's draw (aura.md 5.3): `rect` (`x0, y0, x1, y1` in DDD's virtual screen `SCREEN` = 400 x 240, as slot 2 left it; the whole screen 0, 0, 400, 240), `full_screen` (`+0x19` = 0), `colour` (input 0's output, 0..255, before the element's colour), `blend` (`+0x18`, an index for `0x7ed8e0`), `value` (`v0 + rate t`, never read by DDD's draw). The drawer: one quad, no depth test, colour `clamp(2 x colour x element colour, 0, 255)` through the untextured preset `+0xa4` |
| `TailDraw` / `TailPoint` | FecShapetail's draw (tails.md 1.7): `anm_frame`, `world` (Locus space: false = the part's frame space, draw through `frame()`; true = world), `facing` (Lmatrix byte 1 = 1: screen-aligned; 0: in the frame's plane), `blend` (4 = each quad's code, else 1..3 for `0x7ed8e0`), `orient` (not applied), `reached`; per point `position`, `scale` (factor; x ANM size x 0.1), `angle` (degrees, RotZ), `colour` (0..255, before the element's colour) |
| `Sprite` | one FecMegaanm_plus sprite as its draw evaluates it: `anm_frame`, `flip`, `position` (+ `world`), `scale` (percent), `angle` (degrees; drawn turned by -angle), `colour` (after the fade, before the element's colour, unclamped), `age` |
| `Instance::fade()`, `time()`, `length()`, `is_ended()` | the clock, the stop fade factor (`+0x148`) |
| `Instance::spawn_root(part, time)` | spawn a part as a spawn event does (tests; parts of unimplemented generators) |
| `rng::Rng` (+ `XorShift`) | the random source, injected through `Options` |
| `cmd::class_status(id)` | `Implemented`, `DrawData` or `Stub` |

**Matrices.** DDD uses row vectors (`v' = v * M`, `0x63ad40`). A `Mat4` here holds DDD's 16
floats in DDD's order, so glam's `M * v` is DDD's `v * M` and DDD's `A * B` is glam's `B * A`.
`local()` is FecLmatrix's `S * R * T` (after its parent step); `frame()` is the part's frame
(its owner's own frame, else the instance matrix). INFERRED (commands.md 7.2): a drawn part is
placed by `local` then `frame`.

**Units.** Positions in FEP units with y and z as stored (FecLmatrix negates them into the
matrix); `position()` is the raw Pos output. Colour 0..255 (128 = "full", GUESS). Scale is a
factor (the percent x 0.01). Time in FEP frames; DDD's dt is half the game's frame delta
(`0x554fbd`; GUESS: 1.0 = 1/30 s, so 0.5 per 60 fps frame).

## What is implemented

### Container (`parse.rs`, `sig.rs`, `res.rs`) - spec sections 1-6, 9

- Header (magic `FEP`, versions 1 / 9, `0x5ab93a..0x5ab965`; category masked to 4 bits).
- The command signature table: the exe's 158 records (`0x9e7d10`, count `[0x9e7d00]`) are
  embedded in `sig.rs` byte for byte; a file whose table differs is refused (`Error::Signature`).
  Names of the 158 classes, `param_size`, `work_size`.
- Names (section 7), groups (0) with their elements (0x9c bytes: tests, start offset, `+0x30`,
  start delay), special elements (6: offsets only; empty in every file).
- Bodies (section 1): flags, length, kind, stop mode, unknown fields kept; spawn events; parts
  (a zero entry is `None`); command ids; the twelve `+0x28` lists; part groups.
- Parts: every field of README 6.1; slot descriptors with their inputs; the init list; key
  lists; the eight slot lists. Slot indices are checked against the slot count.
- Resources: the 24-byte records of sections 2..5, local bytes (`file + offset`, `size`), the
  local id formula, resident ids decoded to resident.frr tables (`0xF0000000 | (k >> 7) << 26 |
  type << 7 | (k & 0x7f)`), resident.frr's tables (a resident resource runs to the next one's
  start: the frr holds no sizes). Id-0 records give no data (DDD skips them, `0x5b3076`).
- Offsets: read straight from the file, all file-relative; the u16 parameter offsets are
  list-relative (spec 1.3). A zero 32-bit offset under a count reads as zeros, as DDD reads it
  (e.g. spawn refs `0x5b4648`).

### Interpreter (`run.rs`) - spec section 7

| piece | RVA | notes |
|---|---|---|
| creation | `0x5ac210` | clock = -start delay (`0x5ac4eb`), stop time -1, fade 1; refused when start >= length (`0x5ac30c`) or an id >= 158 (`0x5ac2e8`); catch-up: clock - 1, then a time step of start + 1 (`0x5ac7ed`) |
| time step | `0x5b4290` | dt 0 returns at once; looping + not stopping: dt > 10 -> 1 (`0x5b4361`); stop handling (below); every part's clock += dt (`0x5b4400`); a loopable part of a looping, not-stopping body past its loop end gets the wrap flag and never dies; otherwise death when lifetime <= int(time) unless flag bit 0 and children; spawn events while clock < length or on the first frame (`time - dt == 0`); body end: looping -> when told to stop (mode 3) and empty, bit 0 -> when empty, neither -> at once |
| stop request | `0x5b9cd0` | ignored when already stopping; before the clock reaches 0: end at the next step; else stop time = clock; mode 3 also sets "stopping" (loops release) |
| stop modes | `0x5b43a7..0x5b4518` | 0 / 1: fade `+0x148` = 1 - elapsed / 30 (or 15), end once elapsed >= 30 (15) or < 0; 3: play on; other: end at once |
| spawn events | `0x5b45b2..0x5b46a3` | ascending; stop at the first future one; roots with start time clock - frame; a missing event ends the list |
| spawn | `0x5b73d0` | negative ref / empty entry / 0 slots fail; matrices by type (2, 6..9 three; 0xff none; others two); ref bit 13 (states 0x40 / 0x80); wrap flag at spawn (`0x5b75e0`); next key 0 or -1; init; link at the tail of its pass list; parent's child count; frame owner; spawned in phases 8..9 (generators in the update slots) -> its keys up to int(time) at once (`0x5b793c`) |
| init | `0x5b56f0` | per init entry: its block current, slot 0 with the spawn info |
| frame owner | `0x5b77bd..0x5b7935`, `0x5b85a0` | modes 2, 6..8: own frame = parent local x (owner frame or instance matrix), or the instance matrix for a root; mode 2 resets the rotation, mode 7 normalises the axes; 9: identity; others: the parent's owner |
| death | `0x5b82b0` | unforced: marked dead, kept while it has children or (state 0x40) a held ancestor lives; release slots for slots with a block (then cleared); parent's child count; unlink |
| end | `0x5ad370` | every part freed (forced), the instance ended |
| pass-1 update | `0x5b4700` | phase 7: first-update slots once (state 4, `0x5b6061`); keys (`0x5b5040`); phase 8: update slots (`0x5b611a`); local time = part time - current entry's start |
| after-pass | `0x5b5e70` | phase 0xC: slot 7 list (no implemented class overrides slot 7) |
| list walks | `0x5b5f10` | dead parts skipped; parts appended during a walk are visited in the same walk |
| keys | `0x5b5040` -> `0x5b5180` | skipped when this step's dt is 0; a pending wrap: keys up to loop end - 1, rewind, "looped", clock = `(f - end) mod (end - start) + start + frac`; then while the next list's frame <= f: each entry current, slot 1 (new, old, old span or f); `keys_run` (`ctx+0xa0`) per call |
| loop rewind | `0x5b6680`, `0x5b80c0` | next key 0; per loop-list slot: the key by **class id** (spec 7.2 quirk), else the init list; slot 5 (block, loop start - entry start, old block, loop end - old entry start); next key = found + 1 maxed, or -1 (last slot wins) |
| spawn info | `0x580405`, `0x581c43` | flags 7 with position / velocity / acceleration from generators; none from spawn events |

### Command classes (`cmd.rs`) - spec section 8

| class (id) | status | slots | rests on |
|---|---|---|---|
| FecPos, Posmv, Posac (1-3) | implemented | 0, 1, 2, 5, 6 | `0x583570`, `0x583510`, `0x5835c0`, `0x5835e0`; `0x583290`, `0x583150`, `0x583240`, `0x583300`; `0x582e10` (spawn info **added**, w = 1), `0x582d60` -> `0x5afe70`, `0x582d80`, `0x582f90` -> `0x5b04e0`, `0x583080` |
| FecRot, Rotmv, Rotac (4-6) | implemented | 0, 1, 2, 5 | `0x583d00`, `0x583ca0`, `0x583d50`; `0x583a80`..`0x583af0`; `0x583760`, `0x5836b0`, `0x5836e0`, `0x583800` (wrap into [-360, 360]) |
| FecScl, Sclmv, Sclac (7-9) | implemented | 0, 1, 2, 5 | `0x584320`, `0x5842b0`, `0x5843a0`; `0x584120`..`0x5841c0` (INFERRED, unused); `0x583e60`, `0x583d70`, `0x583dd0`, `0x583f30` |
| FecCol, Colmv, Colac (15-17) | implemented | 0, 1, 2, 5 | `0x57f950` (clamp 0..255, `[0x78e78c]`), `0x57f880`, `0x57fa00`; `0x57f670`..`0x57f740` (INFERRED, unused); `0x57f2f0`, `0x57f1d0` -> `0x5b0090`, `0x57f1f0`, `0x57f3e0` -> `0x5b0720` |
| FecLmatrix (14) | implemented, world step partly | 0, 2, 6 | `0x5825c0`, `0x5821b0`: S * R(order) * T after negating y / z (`0x5826a1`), orders by `0x582990` (jump table `0x582afc`), RotX/Y/Z `0x565a20`/`0x55f3e0`/`0x565b50`; world step `0x5b10e0` modes 0, 3, 4; slot 6 `0x582d30` -> `0x5b85a0` |
| FecGenerate (18) | implemented (decoder only) | 0, 1, 2 | `0x582110`, `0x5819f0`, `0x581a20` |
| FecGravity (19) | implemented | 1 | `0x585350` |
| FecRndpos, Rndrot, Rndscl, Rndcol (22-25) | implemented | 1 | `0x5a9510`, `0x5a9ea0`, `0x5aa800`, `0x5a8a80`; offset tables read from the exe: enable / mode / range = Rndcol `0x9e82fc` (30 32 34 / 31 33 35 / 00 10 20), Rndpos `0x9e830c` and Rndrot `0x9e831c` (0c 0e 1c / 0d 0f 1d / 00 10 20), Rndscl `0x9e832c` (0d 0f 1d / 0e 1c 1e / 00 10 20) |
| FecColorblink_parameter (61) | implemented | 0, 1, 2 | `0x5852c0` / `0x584ff0` (reset: phase ends hold A, + fade, + hold B, + fade; output 1), `0x585050` (phase by counter, targets `0x584cd0` with symmetric randoms, clamp [0, 1], fade-0 jumps; counter per update, not per frame); 4 / 5 nothing (generators.md 7.1) |
| FecColorblink (62) | implemented | 0, 1, 2 | `0x584c70` / `0x584af0` (own copy of input 0, never read), `0x584b40` (input 0's output = clamp(it x input 1's, 0, 255), in place) |
| FecLocus_tail_strategy (68) | implemented | 0, 2, 4, 5 | `0x5a2110` (object `0x5a5fc0`, `init` `0x5a6b80`, `reset` `0x5a6d60`), `0x5a62c0` (carry by the frame delta `0x5a5390` unless `+0x28` or `+0x91 & 7` is 3 / 4; history ring; resampling at the spacings; edges `+0x25`), `0x5a21b0` free, `0x5a21d0` reset when `+0x27`; M / W by space, `0x5a6180` (tails.md 1.3, 1.4) |
| FecNull_routegenerator (79) | implemented | 0 | `0x5853d0`: output (0, 0, 0, 0) (INFERRED: `[0xaa3df0]` is never written) |
| FecRadialg_line_particle (100) | draw data + particles | 0, 2, 4 | `0x576610` (pool of `+0x3c`, n `+0x42`), `0x5765b0` -> `0x565870` (emission `0x565126` (tails.md 4.5), cone about +Y `0x565b50` / `0x565a20`, speed / acceleration / gravity, n-point position history, fade over the second half of life); its draw's strips (`0x563c80`) in `DrawCmd::lines` (tails.md 5) |
| FecEllipse_line_particle (104) | draw data + particles | 0, 2, 4 | `0x560070` (pool of `+0x44`), `0x55fa20` (emission `0x55fa6c`, start on a sphere, spin Rs / Ra about Y `0x55f3e0`, fade); its draw's arcs back along the spin (`0x560130`) in `DrawCmd::lines` (tails.md 4) |
| FecBindlocus_polygon (105) | draw data + strip | 0, 1, 2, 4 | `0x59dcf0` (UV start `0x59e4a0`, B-spline `0x59ef10`), `0x59d2a0` (UV without randoms), `0x59d360` (U / V scroll, wrapped by the TEX size, both edges through the B-spline `0x59e730`, colour lerp x input 0 / 128, head fade, UVs by `+0x4c..+0x4e`); its draw (`0x59deb0`) in `DrawCmd::ribbon` (tails.md 2.4-2.7) |
| FecWave_bindlocus_strategy (106) | implemented | 0, 2, 4 | `0x5a82c0` (object `0x5a0570`, `init` `0x5a1cb0`), `0x5a1060` (first update: `reset` `0x5a0a60`; carry by the frame delta unless `+0x0f` or `+0x91 & 7` is 3 / 4; every sample aged by its velocity; the two points through local x frame as the newest; wave re-target every `+0x0e` updates, follow rate, drift), `0x5a8320` free (tails.md 2.1-2.3). Slot 8 (`0x5a8350`, a reset request) is never called in DDD (tails.md 3): not done |
| FecShapetail (80) | draw data + clock | 0, 1, 2, 4 | `0x5a24c0` (sizes 1, rotations 0, random start `+0x44`), `0x5a2200` (random size `+0x45`, rotation `+0x46`), `0x5a2390` (ANM clock, key, loop / end); its draw's per-point values (`0x5a2850`) in `DrawCmd::tail` (tails.md 1.7). Orient `+0x47` (1.8) not done (deviation 17) |
| FecFlash (107) | draw data + rectangle | 0, 1, 2, 4, 5 | `0x596cb0` (value = base = v0, rect 0), `0x596bb0` (base += new v0 - old v0), `0x596be0` (value = base + t rate; rect clipped to 400 x 240 by unsigned compares when `+0x19`, else the whole screen `200 - 200, 120 - 120, 400, 240`), `0x596d40` (free), `0x597160` (base += t_old old rate - t_new rate, then update); its draw (`0x596da0`) in `DrawCmd::flash` (aura.md 5) |
| FecGenerateanchorvertex (150) | implemented | 0, 1, 2, 5 | `0x57fb60`, `0x57faf0` (old block), `0x57fb30`, `0x5802c0`; burst `0x57fb70`: MDL `+0x0e`'s first mesh, s16 position x 2^-15 x record f32 x mesh `+0x24` (centred by record `+0x14` bit 0), VTL `+0x10` list `+0x12`, in order (cursor) or at random, M = the part's local matrix, placed and spawned as Generateanchor (generators.md 5) |
| FecGenerateanchor (151) | implemented | 0, 1, 2, 5 | `0x580340`, `0x5802d0` (fires a pending burst with the **old** block), `0x580310`, `0x580890`; burst `0x580350`; placement `0x5b23b0` (mode 1, normalised direction); child spawn `0x5b7980` |
| FecAnm_plus (152) | draw data + clock | 0, 2 | `0x586dc0` (clock -dt + random start, flip), `0x586c80` (clock, loop, key search); needs the ANM bytes (resident.frr for resident ANMs) |
| FecMegaanm_plus (153) | sprites + draw data | 0, 1, 2, 4, 5 | `0x58df30` (buffer of N = init `+0xf0`, velocity-to-world flag), `0x58cc40` (emission end, life, fades; nothing once looped), `0x58cdd0` (emission with accumulator, interval, chance, full buffer; spawn by shape `0x58d119` / `0x58df00` with `place` `0x58cb70`; ageing, ANM key, FIFO death), `0x58dfe0` (free), `0x58e7f0` (returns 1); the draw's per-sprite values (`0x58e010`) in `DrawCmd::sprites` (docs/fep/draw-sprites.md 5). Shapes 8, 9 (a VTL vertex of an MDL) are a stub (deviation 14) |
| FecMdl_plus (154) | draw data | (0 returns 1, 1 nothing) | `0x58a100`, `0x58a0f0` |
| FecMdlts_plus (155) | draw data + vec2 | 0, 1, 5 | `0x588f60`, `0x588c80` (ac rebase + randoms from `+0x28..` when byte `+0x4a` and not looped), `0x589770` (rewind, value wrapped into [0, 4096)) |

**Stubs** (init succeeds, nothing else; `ClassStatus::Stub`): every other class. Every class
the 19 Flowmotion elements use is now implemented or gives its draw data.

**Update rate.** The Wave strategy's samples and the line particles' lives, motions and spins
advance once per `step` with dt != 0, unscaled by dt, as DDD's do once per pass-1 update
(tails.md 7; INFERRED there, not checked with the game running). DDD PC updates at 60 Hz with
dt 0.5, so a caller matching it steps 0.5 per 1/60 s (the mod does: its catch-up loop steps in
0.5 chunks); steps of 1.0 give the 3DS rate, one sample per FEP frame.

## Deviations from the spec, and why

1. **Loop rewind, not-found case** (spec 7.2 says "falls back to the init list with t_new =
   0"). The code (`0x5b67b0..0x5b67c7`) computes t_new = loop start - the out-entry's start,
   and the out-entry (`r9` of `0x5b80c0`) is the **last entry examined**, which is the found one
   only when the last key list examined holds it; for the init fallback that is the init entry
   (start 0), so t_new = loop start. The crate follows the code, quirk included
   (`run::loop_key`).
2. **Signature check** at parse time over the whole table, not per used id at element creation
   (`0x5ac2c2`). Same verdict on every file (667/667 carry the exe's table); simpler.
3. **Creation phase.** DDD's catch-up depends on the phase the context is in (`0x5ac74c`):
   0 subtracts dt and ignores the start frame, 1 / 5 / 9 run a time step of start + 1. `start`
   behaves as phases 1 / 5 (creation between the time task and the pass-1 update, then that
   frame's pass-1 update). INFERRED: motion events create effects there. If DDD creates them
   elsewhere (phase 0xC, after the draw prep), its first frame would be one step later.
4. **Pass 0** (part byte `+0x09` bit 0; no file has such a part) is run like pass 1, before it
   (GUESS; `0x5b53e0` was not read).
5. **Not modelled** (they gate DDD's updates or draws but are about the scene, not the
   effect): the category masks (`0x5b4311`, `0x5b4717`), pause flags `+0x0c` / `+0x0e`, layers
   `+0x1b5`, the view and distance tests (`0x5b48e9..0x5b4d31`), context bit 0 (frozen time,
   `0x5b61b8`), the pending extra time `+0x140`, the start-delay countdown `+0x168` (always 0 at
   creation, `0x5ac3a6`), the draw-state sort (`0x5bc450`, Lmatrix bytes 4 / 5), the draw passes
   by `0x9e7c60` bit 0 and the draw-queue limit (`0x5e19b0() >= 0x50`).
6. **Instance matrix.** `0x5ae260` (position / rotation / scale + attachment -> `+0x60`) is not
   decoded; the caller sets the matrix (`set_matrix`). The element's start offset (`+0x10`,
   added to the start position at `0x5ac67b`) is parsed but not applied: the caller adds it.
7. **World matrix step** `0x5b10e0`: modes 0, 3, 4 as read; the scale argument is always 1.0
   from FecLmatrix (`0x582719`). **TODO**: `0x5b1510` (a part without a draw state whose `+0x91 &
   7` is 3 or 4 has its matrix combined with its frame; not every branch followed), and
   FecLmatrix byte 2 (orient along the velocity, `0x5822fd..0x582493`; 0 in every file). Mode 4's
   3x3 product order (`0x5b2de0`) is INFERRED from its callers.
8. **Init failure.** No class here fails slot 0, so DDD's clean-up (`0x5b57e8`, which calls the
   failing slot's release i + 1 times) is not reproduced.
9. **Generator part groups** (child ref bit 12, `0x5806a0..0x5807f9`, `0x5b0e80`) are not
   followed: no file uses one, and the groups hold file-wide part numbers (spec 5).
10. **Randomness.** DDD draws from one game-wide generator (`0x613020` -> `0x6196a0`, INFERRED
    a Mersenne Twister); its sequence cannot be reproduced, so commands draw from `rng::Rng`
    (default xorshift64*). Each draw is turned into a number exactly as DDD does (`(r >> 1) *
    2^-31`, `[0x7ecf78]`), and the number and order of draws follow DDD's code (e.g. three per
    enabled Rnd* range, x then y then z).
11. **Guards where DDD would fault or hang** (never reached by the files): an empty loop
    (`end == start`, DDD `idiv` by zero, `0x5b510b`) wraps to the loop start; an ANM of length 0
    ends; the Rotmv/Rotac [-360, 360] wrap first reduces huge values; a frame capture into a part
    without an own frame matrix (DDD writes through null) is skipped; reads past a work area or
    the file give 0, writes are dropped.
12. **Part storage.** Part instances live in a slab with reused slots; indices stay valid
    because DDD never frees a part with live children (`0x5b82e5`), so a parent and an owner
    outlive their children. Dead parts kept for their children are not listed by `parts()`
    (DDD's draw skips them, `0x5b59ea`).
13. **Release** slots: only FecMegaanm_plus overrides slot 4 (frees its buffer). Its buffer is a
    `Vec` of records in the part instance, not bytes in the work area; DDD's allocation is not
    zeroed (`0x5bd400`): the header fields only a key writes (emission end, life, fades) start
    at 0 here (every Flowmotion part keys its block at frame 0), and N = 0 gives an empty
    buffer (DDD would divide by zero in its ring index).
14. **FecMegaanm_plus shapes 8 and 9** (spawn at a VTL vertex of an MDL, as
    FecGenerateanchorvertex, `0x58d370..0x58d4b4`) are not implemented: they take DDD's
    "missing piece" path (position 0, radial direction 0, no random draw for the vertex, the
    in-order cursor `+0x10` untouched). None of the 49 Flowmotion blocks uses them. A looping
    ANM of length 0 (DDD: `idiv` by zero, `0x58ddd1`) kills the sprite. `Sprite::colour` is
    not clamped (DDD clamps after multiplying the element's colour, which the caller has).

15. **World-space trail units** (not DDD's): a FecLocus_tail_strategy in world space (`+0x26`,
    COM LOC0) records local x frame and measures its spacing there. DDD's frames are in DDD
    units; a caller's may not be (the mod's carry DDD -> KH1, x100). `Options::world_unit` (1 =
    DDD) multiplies a world-space Locus's spacings; part-space ones are unaffected.
16. **Locus frame delta** (tails.md 1.3) as written: `D = inv(M_old) W_old inv(W_new) M_new`
    (DDD order). It is the identity whenever W = M, which holds for every Flowmotion trail here
    (our world matrix equals the local one for parts of type 0 and 1; `0x5b1510` is not decoded,
    deviation 7), so a trail stays in its sample space (frame space for space 0, world for space
    1). tails.md's remark "with W = M this is inv(M_old) M_new" contradicts its formula; that
    reading would carry every recorded point rigidly with the part and never leave a trail.
17. **FecShapetail.** Orient (`+0x47`, tails.md 1.8: helper `0x5b39a0` / `0x5a3420`, depth offset
    `0x5baa40`) is not done: `TailDraw::orient` passes the vector, nothing applies it (COM LOC0
    only). D for Lmatrix byte 1 = 0 is `0x5b2670` (not read; INFERRED the frame); both Flowmotion
    trails use byte 1 = 1. Slot 1 draws all sizes, then all rotations (INFERRED order). The clock
    starts at 0 and the spawn frame's update already adds dt, as slot 2 does.
18. **FecGenerateanchorvertex**: a list of 0 vertices (never in the data; DDD would read past it)
    fires nothing.
19. **Ribbons and line particles** (tails.md 2-5):
    - The element's colour (instance `+0x40`) is taken as 1: the drawer multiplies it after the
      crate's clamps (DDD multiplies it into input 0 first; the same while it is 1, as in every
      Flowmotion EAD event).
    - FecBindlocus_polygon: the cubic spline kind (`+0x53` = 0, `0x59f1b0`, not decoded; unused
      in Flowmotion) draws the raw samples; another kind (DDD: the spawn fails) draws nothing. The
      depth offset is passed raw (`RibbonDraw::depth`; `0x5b39a0` / `0x5baa40` not reproduced);
      the viewport reset `+0x4f` is not modelled. A missing texture leaves U / V where they are.
    - Wave_bindlocus_strategy slot 8 (a reset request) is not done: DDD never calls it (tails.md 3).
    - Line particles: a life of 0 (DDD: the spawn fails) makes no pool and draws nothing; a new
      particle takes the first free slot at or after the one taken before it in the same update
      (INFERRED for DDD's running index); colours become bytes by truncation (INFERRED);
      `[0x7b1208]` is taken as -pi/180. A life that comes out 0 from its random wraps to 65535
      updates, as DDD's u16 does.
20. **FecFlash** (aura.md 5): the virtual screen is DDD's static 400 x 240 (the PC rewrites
    `[0xaa6e28]` / `[0xaa6e2c]` for its aspect, `0x60ef75`: the drawer maps the screen onto its
    own); the draw's skip on element instance `+0x1b6` is not modelled. The rect's s16 fields keep
    DDD's u16 truncation.

## Checks (`tests/flowmotion.rs`, on D:/KH-Extract/ddd/original; skipped when missing)

- the seven Flowmotion FEPs parse; every name resolves; local ids and magics; resident.frr
  tables (32 / 134 / 136 / 24) and every resident id the seven files use;
- per body and part: length, part count, lifetime, slot count, key-list count, loop start /
  end, matrix type equal `tools/ddd_fep_dump.py`'s dumps (all 42 bodies, 355 parts);
- the 19 Flowmotion elements (spec 11.1; body, length, looping, stop mode, part count checked)
  start and run to their end at dt 1, 0.5 and 0.25 (looping ones are stopped after three
  lengths), with finite matrices;
- spec 8.1's numbers: p_fa_ajc0 element 0 part 2 Sclac 2.00 at frame 0 -> 3.41042 at its key at
  frame 9 -> 2.16333 at frame 15, and the curve between; part 1 Colac 160 -> 128 at frame 18;
- FecMegaanm_plus on draw-sprites.md 5.6's examples, at dt 1: dive SHO0 part 11 (box, y
  acceleration 0.008 +- 0.002, emissions at frames 0-5 and 8, life 13), part 17 and slam SHO0
  part 14 (rings of radius 4 / 1.3: radius `R + 0.15 a - 0.005 a^2`, z `-0.05 a + 0.0005 a^2`,
  fade in under age 1 and out over the last 3 frames, spin 70 a frame, flips), slam part 15
  (N 5 full: frame 10's sprite is lost); every frame's ages, oldest first, match the
  emission schedule (ages `r / s` after the first update), and the buffers empty;
- FecGenerateanchorvertex: the dive's part 19 spawns its five part-18 trails at once, on the
  pentagon corners of generators.md 5.5 through part 19's local matrix, and never again;
- the dive's trail (part 18): Locus N 17, the newest point at the part's origin, chords <= 0.12,
  V grows to 17; Shapetail: points = V, the head colour = Colac x (160, 110, 120, 128) / 128, the
  last point's size and colour (V-1)/V of the way to the tail, the ANM key from the clock. The
  rail trail (COM LOC0, world space) with a moving instance matrix: 17 points on its line, 0.24 /
  0.10 apart behind the newest;
- the Keyblade ribbon (w_010_so LOC0) with the matrix moving 0.1 an update: 58 vertices, the
  strip's ends at the newest and the 6th-newest samples of A (0, 0, -0.735) and B (0, 0, 0.0189),
  U 0 / 0.5, V 1 - k/29, colour = the Colac lerped to 0 over 29 points, alpha x 0 and x 0.5 at the
  first two points (head fade); AJC / AKD CAL0: 122 vertices, flipped U, V scrolling 2.5 texels an
  update over 64, head colour x Col / 128, the strip unbroken;
- ASK MLC0's ellipse lines: a particle every 2 updates at dt 0.5, at most 9 alive, 3-point arcs
  on a circle about Y, 7..13 degrees apart, colour in thirds; COM BSS0's radial lines: one
  segment of 0.2 inside the 40-degree cone, colours (1, 0.75, 0.25) / (1, 0.5, 0.25) x Col;
- Colorblink: COM BSS0 part 2's factor alpha 0.75, 0.5, 0.75, 1.0, ... applied to its Colac in
  place; part 12 alternates B, A from the first update;
- the dive's rings (SHO0 parts 12..15) appear as children of part 16; a stop on a stop-mode-0
  element fades 1/30 a frame and ends after 30; start frame 12 has fired the frame-12 events;
  truncated and altered files are errors;
- FecFlash: COM FSH0 part 10 draws the whole screen (0, 0, 400, 240; its block's 512 x 448 rect
  unused), blend selector 1, value 128, colour (128, 64, 128) with alpha `8 + 59.72 t - 15.86
  t^2` to 64 at its key at frame 2, nothing from frame 7;
- ignored: all 667 FEPs under `effect/` parse and all 4145 names resolve.
