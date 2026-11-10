# FEP space and time: where and how fast DDD plays an EAD-started effect

Scope: the path from an EAD event (a motion's effect event) to the FEP element instance's
matrix (`+0x60`, built by `0x5ae260`), what stops such an effect, the FEP clock, and the units.
Exe: DDD Steam (KH 2.8), RVAs (image base 0x140000000). Data: `p_ex01_000.ead` (Sora),
`w_so01_001.ead` (Keyblade), `effect/sys/ffa/bin/*.fep`.
**Labels:** unmarked = read in the exe this session (RVA given); **INFERRED** = follows from code
read but a link was not traced; **GUESS** = not read. Matrices in DDD's own **row-vector** form
(`v' = v · M`, translation in row 3) unless a line says "column form".

Tools: `tools/ddd_fep_triggers_ead.py` (EAD dump, fields), `tools/ddd_fep_space_ead.py` (placement
case and stop rule per event), `tools/ddd_fep_space_clock.py` (clock
constants, game-speed setters), `tools/ddd_fep_space_parts.py` (element placement data),
`tools/ddd_fep_space_emu.py` (what the matrix routines compute; its verdicts are cited below).

---

## 1. The objects

| object | what | created / read |
|---|---|---|
| EAD event | 52 bytes, section 2 | read by `0x5545a0` (per-frame start/end test) and `0x5521e0` (start) |
| event state | 0x18 bytes per event of the current motion: `+0` u16 bit 0 = fired this lap, `+8` -> event, `+0x10` starts so far, `+0x14` handle id | built by `0x552b40` when a motion starts (`0x553c60` and siblings) |
| effect record | 12 bytes per started effect: `+0` handle id, `+4` s16, `+6` change-fade frames (event `+0x2a & 0xfff`), `+8` flags (event `+0x30 & 0xfff`), `+0xa` event `+0x32` | written by `0x552990` (`0x552aec..0x552b1b`); used by the motion-change handler `0x552de0` |
| handle | 0xc0 bytes (`0x54f2b0`), one per started effect: `+0x08` colour vec4, `+0x18` EAD scale vec3, `+0x50` -> its node object, `+0x58` -> FEP element instance, `+0x60` flags, `+0x68` alpha, `+0x6c` alpha rate, `+0xb8` stop delay | `0x5559a0` -> `0x54fc80` |
| effect node | 0xb0 bytes, KH-style scene node: `+0x08` local 4x3 (3x3 rows at `+0x08/+0x14/+0x20`, translation `+0x2c`), `+0x38` world 4x3 (rows `+0x38/+0x44/+0x50`, translation `+0x5c`), `+0x68` -> parent node, `+0x70` bit 0 dirty | `[handle+0x50]+0x10`, entry 0 |
| bone node | the same 0xb0-byte node in the owner's skeleton: `[owner+0x100]+0x10 + index·0xb0` (index from the bone name, `0x5f3000`) | `0x5529e3..0x552a20` |
| element instance | FEP instance (README 7.3): `+0xa0` position, `+0xb0` rotation (deg), `+0xc0` scale, `+0xd0` colour, `+0xe0..+0x11f` 4x4 basis, `+0x1b4` rotation order | setter `0x5ae740` |

**The setter `0x5ae740`** (inst, block): block `+0` u32 mask, `+8` u8 order, `+0x10..+0x30`
pointers. Mask 1 -> `+0xa0` (vec4), 2 -> `+0xb0`, 4 -> `+0xc0`, 8 -> `+0xd0`, 0x10 -> `+0xe0..+0x11f`
(4x4), 0x20 -> byte `+0x1b4` (`0x5ae7c1..0x5ae84b`). An instance whose first byte is 0x6c ('l', a
list) passes the block to each child. **Creation** (`0x5ac5cf..0x5ac663`) sets mask 0x3003f:
position (0,0,0,1), rotation 0, scale 1, colour 1, basis identity, order 0. The EAD path never sets
mask 2 or 0x20, so for these effects `+0xb0 = 0` and `+0x1b4 = 0` throughout.

---

## 2. The EAD event (52 bytes): every field

| off | type | meaning | where |
|---|---|---|---|
| 0x00 | u32 | -> effect file name | `0x5522e9` |
| 0x04 | u32 | -> element name | `0x552344`, `0x55242a` |
| 0x08 | u32 | -> bone name; resolved in the owner's skeleton; not found -> no bone (world origin, see 3.4) | `0x5529d4..0x552a20` |
| 0x0c | s16 x3 | **offset**, x 0.01 = DDD units (`[0x867d08]` = 0.01), in the **bone's local frame** (3.2). Raw value = KH1 units (KH1 = DDD x 100) | `0x551b90` |
| 0x12 | s16 | effect group filter (event runs when the motion's group < 0, this < 0, or they are equal; triggers tool, spot-checked: read at `0x55412f`, `0x55415d`) | `0x554070` |
| 0x14 | u8 x3 | tint RGB, used when non-zero (`0x5524a4` -> handle `+0x78..+0x7a`) | `0x5524a4` |
| 0x17 | u8 | tint ramp-in frames (`0x5505c0`) | `0x5524d4` |
| 0x18 | s16 x3 | **rotation**, x 0.0001 = **radians** (`[0x867ce4]` = 1e-4), in the bone's frame, applied **X first, then Y, then Z** (row form `RX·RY·RZ`; 3.2) | `0x551be0` |
| 0x1e | s16 x3 | **scale**, x 0.01 (1.0 = 100) -> handle `+0x18` | `0x551c30`, `0x552491` |
| 0x24 | u8 x4 | colour RGBA x 1/128 (`[0x7a94a0]` = 0.0078125) -> handle `+0x08`: 0x80 = 1.0 | `0x550c00` |
| 0x28 | u16 | **start time** t0, motion time in 1/60 s (4.1) | `0x554722` |
| 0x2a | u16 | low 12 bits: fade frames on a motion change (record `+6`); high 4 bits not read | `0x552afe` |
| 0x2c | u16 | **end time** t1 (0 = none) | `0x554656`, `0x554687` |
| 0x2e | u8 | fade-out frames at the end time | `0x5546df` |
| 0x2f | u8 | fade-in frames (`0x550580`, less the time already late) | `0x55254f` |
| 0x30 | u16 | flags (below) | |
| 0x32 | u8 | tint ramp-out frames (passed as the second time to `0x550610`/`0x550af0` -> handle `+0x7b`) | `0x552b16`, `0x5546cb` |
| 0x33 | u8 | max starts per motion lap (0 = no limit; event state `+0x10`) | `0x55470e` |

**Flags `+0x30`:**

| bit | meaning | where |
|---|---|---|
| 0-1 | motion-change mode: 0 fade over `+0x2a`, 1 stop request now, 2 keep, 3 stop request after `+0x2a` (5.2) | `0x552e48..0x552e86` |
| 2 | **follow the bone**: the effect node is parented to the bone node (3.2) | `0x552332..0x552383` |
| 3 | **take the bone's rotation**; clear -> handle `+0x60 \|= 0x100`: the effect keeps the EAD rotation in world axes (3.3) | `0x55238d`, `0x552524..0x552528` |
| 4 | at the end time: FEP stop request (else fade the alpha over `+0x2e`) | `0x5546d9` |
| 5 | handle `+0x60 \|= 0x80`: scale = EAD scale alone, not times the node's scale | `0x552530`, `0x551685` |
| 6-7 | == 1: on a motion change, fade even a non-looping effect | `0x552e8d..0x552e93` |
| 8 | handle `|= 0x40` and owner link (`0x550c80`): alpha x the owner's `+0x5c` (`0x550f45`) | `0x5524e4..0x5524ff` |
| 9 | start test on the unwrapped motion clock (`0x5c88b0`) | `0x55472d` |
| 10 | end test on the unwrapped motion clock | `0x554692` |
| 11 | handle `|= 0x20` and owner link: colour x the owner's `+0x50` (`0x550f4f`) | `0x552507..0x55251c` |
| 12-15 | level: the event is skipped when it is above the level passed in | `0x5529c2..0x5529ce` |

### 2.1 Motions 322 (slam) and 323 (dive), fully decoded

`python -I tools/ddd_fep_triggers_ead.py --motions 322,323 --raw`. Times: EAD ticks (1/60 s of
motion time) and the motion frame (= tick / 2 for these 30 fps motions).

| motion | t0 (frame) | t1 | file / element | bone | offset (DDD) | rot | scale | RGBA | flags | placement (3.3) |
|---|---|---|---|---|---|---|---|---|---|---|
| 322 | 6 (3) | - | p_fa_akd0 / P_FA_AKD_CAL0_0 | `mune` | 0 | 0 | 1 | 80808080 | 0x0004: follow, no bone rotation, mode fade | case B |
| 322 | 54 (27) | - | p_fa_akd0 / P_FA_AKD_SHO0_0 | `Root` | 0 | 0 | 1 | 80808080 | 0x0000: stay, no bone rotation | case D |
| 322 | 42 (21) | 50 (25) | w_010_so / W_010_SO_LOC0_0 (Keyblade EAD) | `Z_00` | 0 | 0 | 1 | 80808080 | 0x001c: follow + bone rotation, end = stop request | case A |
| 323 | 4 (2) | - | p_fa_ajc0 / P_FA_AJC_CAL0_0 | `mune` | 0 | 0 | 1 | 80808080 | 0x0004 | case B |
| 323 | 64 (32) | - | p_fa_ajc0 / P_FA_AJC_SHO0_0 | `Root` | **(0, 0, 1.30)** | 0 | 1 | 80808080 | 0x0008: stay, bone rotation | case C |
| 323 | 52 (26) | 68 (34) | w_010_so / W_010_SO_LOC0_0 (Keyblade EAD) | `Z_00` | 0 | 0 | 1 | 80808080 | 0x001c | case A |

Every other field of these six events is 0: group 230 (slam) / 235 (dive) / -1 (Keyblade), no
tint, change-fade 0 for Sora's four and **8** for the two Keyblade events, end fade 0, fade-in 0,
`+0x32` 0, max 0, level 0.

---

## 3. Placement: from the event to the instance matrix

### 3.1 Start (`0x5521e0`, called by `0x552990` from the per-frame test `0x5545a0`)

1. `off = event+0x0c x 0.01`, `rot = event+0x18 x 1e-4` rad (`0x55224b`, `0x552258`).
2. **Follow** (bit 2, bone found): `0x5563b0 -> 0x5559a0(..., parent = bone node, pos = &off,
   rot = &rot)`. The new effect node gets `local = [R(rot) | off]`, `R = RX·RY·RZ` from radians
   (`0x63c5f0` RotX, `0x63c6b0` RotY, `0x63c760` RotZ, products by `0x63ac90` at `0x555b8b`,
   `0x555b9e`; emu: these are row-form 4x4 rotations), and its parent link
   (`0x5503c0`: node `+0x68` = bone node, handle `+0x60 |= 0x10`).
3. **Stay** (bit 2 clear, or no bone): base = the bone node's world 4x3 (bit 3 set,
   `0x5523d3`) or identity with the bone's world position (bit 3 clear, `0x552392..0x5523ce`;
   identity when there is no bone). `O = [RX·RY·RZ(rot) | off]` (`0x200340`, emu: RX·RY·RZ),
   `local = O · base` (`0x12ca90`, emu: B·A = O·base). `0x558b90 -> 0x5559a0(..., matrix =
   &local)`: no parent, so the node's world = this local, **fixed for the effect's life**.
4. Handle setup: EAD scale -> `+0x18` (`0x552482`), colour (`0x550c00`), tint, flags bits
   8/11/3/5 -> handle `+0x60` bits 0x40/0x20/0x100/0x80 (`0x5524e4..0x552534`).
5. **Late start**: `late = motion time - t0` (`0x5547c1`). If `late > 0.01`: `0x550db0` runs the
   FEP element `int(late x 0.5)` frames ahead (`0x550e71..0x550e8c`, `0x5af030`), or kills a
   non-looping one whose `late >= 2 x 0x5af5d0()` (INFERRED: its length). Fade-in = `+0x2f - late`.

### 3.2 Every frame: node -> instance (`0x554ed0`, task priority 0x543A8, `0x557ee4`)

For every handle not dead (`+0x60` bit 15 clear): `0x551090` (placement), `0x551880` (pause,
timers: `0x5500a0`), `0x550ef0` (colour).

**`0x551090`** (node = entry 0 of `[handle+0x50]+0x10`):
- Parent gone: when the handle is attached (`+0x60 & 0x10`) and no node has a parent any more,
  it returns 0 and the caller kills the effect (`0x5510bf..0x551100`, `0x554f32..0x554f39`).
- `M = node world rows (+0x38)`, or **node local rows (+0x08) when `+0x60 & 0x100`** (bit 3 of
  the event clear) (`0x551134`, `0x551279..0x5512b9`). Row lengths `len_k` (`0x63b640` + sqrtf;
  <= 0 -> 0.001, tiny -> 1e-5).
- **Basis**: rows of M divided by `len_k`, translation 0 -> instance `+0xe0` (mask 0x10,
  `0x5513c2..0x5514bb`).
- **Position**: node world translation `+0x5c` -> instance `+0xa0` (mask 1, `0x55160a..0x551680`).
  (With handle bit 0x200 and a parent: `local translation · M + parent world translation`
  instead, `0x5514d6..0x551608`; nothing in `0x54f000..0x559000` sets 0x200.)
- **Scale**: EAD scale `x len_k` (or the EAD scale alone with `+0x60 & 0x80`) -> instance `+0xc0`
  (mask 4, `0x551685..0x551833`).
- Marks the node dirty (`+0x70 |= 1`, `0x55185a`).

The node's world is `local · parent world`, recomputed by DDD's node code (`0x5cfb00` ->
`0x5ce9a0`, frame-stamped, dirty bit `+0x70`) (**INFERRED** that it runs every frame for these
nodes and before `0x554ed0`; the caller was not traced). So a **following** effect's node world
moves with the bone every frame; a **staying** one's never changes.

### 3.3 The four cases (for Sora's Flowmotion events)

`W_b` = bone node world (rotation rows incl. any scale, translation `t_b`); `R_e = RX·RY·RZ(rot)`;
`s_e` = EAD scale; `nrm()` = rows normalised.

| case | flags | node world | instance `+0xe0` (E) | `+0xa0` (P) | `+0xc0` (Sc) | moves? |
|---|---|---|---|---|---|---|
| A follow + bone rot | 4 \| 8 | `[R_e \| off] · W_b(now)` | `nrm(R_e · W_b)` | `off · W_b + t_b` (now) | `s_e x row lengths of R_e·W_b` | yes, every frame |
| B follow, no bone rot | 4 | same | **`R_e`** (from local, world axes) | `off · W_b + t_b` (now) | `s_e` | position only |
| C stay + bone rot | 8 | `[R_e \| off] · W_b(t0)` | `nrm(R_e · W_b(t0))` | `off · W_b(t0) + t_b(t0)` | `s_e x row lengths` | no |
| D stay, no bone rot | 0 | `[R_e \| off + t_b(t0)]` | **`R_e`** | `off + t_b(t0)` (offset in world axes) | `s_e` | no |

So: CAL0 (case B) rides the chest's position with world-aligned axes; the dive SHO0 (case C) is
placed once at 1.30 DDD units along `Root`'s +z (forward, **INFERRED**: Root's local z = the
model's forward, ddd.rs axes) with Root's rotation; the slam SHO0 (case D) is placed once at
`Root`'s position with world axes; the Keyblade LOC0 (case A) follows `Z_00` fully.

### 3.4 The instance matrix (`0x5ae260`, every pass-1 update, README 7.1)

With `el` = the element (`inst+0x1a8`; the null-element branch `0x5ae57c` is not used here):

```
inst+0x10 = el+0x10 ⊙ Sc + P            0x5ae294..0x5ae31d   (0x63b890 = component multiply)
inst+0x20 = inst+0xb0 + el+0x20  (deg)   0x5ae322..0x5ae382   (+0xb0 = 0 for EAD effects)
inst+0x30 = el+0x30 ⊙ Sc                 0x5ae2b4, 0x5ae387
M3 = diag(inst+0x30) · RZ·RY·RX(inst+0x20, degrees) · E3     0x5ae38c..0x5ae4c7
     (order byte +0x1b4 = 0 -> 0x5af8b0 order 0 = In·RZ·RY·RX, emu; E3 = rows +0xe0/+0xf0/+0x100;
      0x63a950 = B·A, emu)
inst+0x60 rows 0..2 = M3; row 3 (+0x90) = inst+0x10, +0x9c = 1.0   0x5ae4de..0x5ae61e
inst+0x40 = inst+0xd0 (colour)            0x5ae632, 0x5ae661
```

The basis' own translation row (`+0x110`) is not used. `el+0x10` is a world-axis offset scaled by
`Sc` (not rotated); `el+0x20` is a rotation in degrees applied before E; **`el+0x30` is a scale**
(corrects README section 4, "no reader seen"). For all 19 Flowmotion elements `el+0x10 = 0`,
`el+0x20 = 0`, `el+0x30 = 1` except P_FA_COM_FIR0_0 (1.2) (`ddd_fep_space_parts.py`).

**Column form** (for the KH1 side): `A = T(P + el.off ⊙ Sc) · R_node · Rx(el.rx)·Ry(el.ry)·Rz(el.rz)
· diag(el.scl ⊙ Sc)`, where `R_node` = the column-form rotation whose columns are E's rows.
For Flowmotion: **`A = T(P) · R_node · diag(el.scl ⊙ Sc)`**.

---

## 4. Time

### 4.1 Clocks

- **Frame step** (`0x540a2b..0x540a72`): elapsed microseconds x `[0x7e9dc8]` = 5.994e-5, i.e.
  **1.0 per 1/59.94 s** whatever the frame rate (clamped to a maximum, `0x540a4d`; another branch
  `0x540a5e` counts whole ticks). Stored at `[0xa9b9e0]` (`0x540ace`, read by `0x540760`).
- **Game speed** (`0x53edc0`): `dt = step x [0x9e6be8]` (default 1.0; set by `0x53edf0`), 0 when
  below 1e-4. Stored at `[0xa9b9dc]` (`0x540af6`), read by `0x540770`. The unscaled step goes to
  `[0xa9b9d8]` (`0x540ae9`, read by `0x540750`) (flag bit 1 at `0x540ad6` substitutes a constant).
- **`0x6294e0(dl)`** = `[rcx+0x18] -> 0x629ca0`: dl != 0 -> `0x540770` (dt), else `0x540750`.
  The FEP time task (`0x554f80`, priority 0xEA60) calls it with dl = 1.
- **FEP**: `dt_fep = 0.5 x dt` (`0x554fd4`), `dt2_fep = 0.5 x unscaled step` (`0x554fbd`); both 0
  when `0x60e160()` (README 7.1). An instance takes dt unless its `+0x0a`/`+0x0b` say otherwise;
  creation clears both (`0x5ac4c6`), so EAD effects run on **dt** (**INFERRED**: no EAD path
  sets them).
- **So one FEP frame = 2 ticks = 1/29.97 s at game speed 1: 29.97 FEP frames per second**
  (the README's GUESS 30, now read). At 60 fps DDD steps 0.5 per frame.
- **Handle timers** (fade-in, fade-out, stop delay; `0x5500a0`) run on `[0xaa3cf8]` = the same
  speed-scaled `dt` before halving (`0x554fcc`, `0x554ee2`): their frame counts (`+0x2a`,
  `+0x2e`, `+0x2f`) are **ticks (1/60 s)**.
- **EAD clock**: `0x5c8eb0` = `motion frame (+0x5c) x 60 / motion fps (+0x54)` (`[0x867f04]` =
  60): **1/60 s of motion time**, i.e. twice the frame of a 30 fps motion (sound.rs's rule).
  Start: fires when the motion time reaches t0 (with wrap handling, `0x55475d..0x5547b1`); end
  likewise (`0x554656..0x5546c5`).
- Order (tasks sorted by priority, README 7.1 caveat): FEP time 0xEA60 -> pass 0 0x249F0 ->
  handle placement 0x543A8 -> `0x5550b0` 0x7EF40 (`0x557eb1..0x557efa`); pass 1 (`0x5ae260`) is
  reached from `0x5dffc0` (render side). **INFERRED**: A is this frame's node world.

### 4.2 Flowmotion slow-downs

- The FEP clock sees only the **global** game speed `[0x9e6be8]`. Per-motion speed (the motion
  clock) does not reach it: a slowed motion delays the **triggers** (EAD times are motion time)
  but an effect already running plays at the global rate.
- Global speed setters (`ddd_fep_space_clock.py`): 18 calls set 1.0, two set **0.2** (`0x1df6e3`
  with a 40-tick timer `[rbx+0x410]`, `0x20f4f7`), 19 pass a variable.
- The slam (322) is recorded running 0.1-0.2 motion frames a tick at its frames 4.5-8.5
  (ddd-notes.md) where 0.5 is normal: 0.2-0.4 x speed. **Not traced** whether this is the global
  speed (then the FEP and every handle timer slow too) or the motion's own speed (then they do
  not). 0.1 = 0.5 x 0.2 matches the 0.2 setters, which is suggestive only. **Open.**
- The dive (323) shows no slow-down in the recordings (its hang is root motion at normal clock).

---

## 5. What stops an EAD-started effect

### 5.1 End time (`0x5545a0`, `0x554656..0x5546ff`)

When the motion time reaches `t1` (> 0) and the handle is alive: bit 4 set -> **stop request**
`0x550af0(h, 0, +0x32)` -> `0x5b9cd0`: instance `+0x144 = time` (or `+2 |= 0x20` while still in its
start delay); body `+0x0c == 3` also sets `+2 |= 0x10` (`0x5b9d38..0x5b9d6d`). The body's mode
then decides (README 5; `0x5b43a7..0x5b4528`): 0 / 1 -> fade factor `+0x148 = 1 - (time - stop
time) / 30` (or / 15) and removal once 30 (15) FEP frames have passed; other (not 3) -> removed at
once; **3 -> no fade, but state `+2 & 0x10` stops every part from looping (`0x5b442e`) and lets a
looping body end as soon as its pass lists are empty (`0x5b44b8..0x5b44d6`)**: it finishes
naturally. Bit 4 clear -> `0x550610(h, +0x2e, +0x32)`:
fade the handle alpha to 0 over `+0x2e` ticks, then kill (0 -> kill at once). The event state then
forgets the handle (`0x5546ff`).

### 5.2 Motion change (`0x552de0`, first thing in `0x553c60` and its siblings when a motion starts)

For each effect record of the previous motion, with `looping = body flags bit 1`
(`0x550a20 -> 0x5ae110`):
- looping: mode 0 -> `0x550610(h, +0x2a, +0x32)` fade (0 frames -> kill at once); 1 -> stop
  request now; 2 -> nothing; 3 -> `0x550af0(h, +0x2a, +0x32)`: stop request after `+0x2a` ticks
  (handle `+0xb8`, bit 25; fired in `0x5500a0`, `0x550109..0x55017a`).
- not looping: nothing, unless flags bits 6-7 == 1 (then fade as mode 0).
- Then the record table is emptied (`+0x58 = 0`) and the new motion's event states are built
  (`0x552b40`): **the old motion's end times no longer apply**.

### 5.3 Other ends

- The FEP element ends by itself at its body length (README 5).
- Parent gone (3.2) or the pause/timer check failing (`0x551880 -> 0x5500a0` returns 0 when a
  fade-out reaches alpha 0): `0x550a90` -> handle `|= 0x8000`, `0x550440(h, -1)`, `0x54f860` ->
  `0x5ad370` (instance removed at once).
- Fade is the **handle alpha** (`+0x68`): `0x550ef0` multiplies the colour's w by it and sets it as
  the instance colour (mask 8 -> `+0xd0` -> `+0x40` in `0x5ae260`), and hides the instance at
  alpha 0 (`0x5b9000(..., 5)`). How `+0x40` reaches the draws is not decoded (README 13.6).

### 5.4 The dive and slam

Sora's four events are all mode 0, bits 6-7 = 0, no end time, and every one of their bodies is
non-looping (flags 0: AJC CAL0 35 frames, AJC SHO0 45, AKD CAL0 30, AKD SHO0 45). **Nothing in the
EAD stops them**: each plays to its own length (1.17 s, 1.5 s, 1.0 s, 1.5 s at 29.97 FEP frames/s)
even after the motion changes, and is removed at its length (body flags 0). The Keyblade LOC0
(looping, 6 frames, stop mode 3) gets a stop request at t1 (motion frame 25 / 34): its parts stop
looping and it ends when they have died; on an earlier motion change it gets an 8-tick alpha fade
then removal (mode 0, `+0x2a` = 8).

`python -I tools/ddd_fep_space_ead.py [--motions 322,323]` prints this per event. Over all 82
Flowmotion-element events: case A 68 (the 45 Keyblade `Z_00` events and 23 of Sora's on `Root`,
`L_ashi2`, `hara`, `center`), B 3 (both CAL0 on `mune`, COM LOC2 on `L_ashi2`), C 5 (AJC SHO0,
COM FIR0, COM WAL0 x3, all `Root`), D 6 (AKD SHO0 on `Root`, AC RNG0 x5 on `mune`).

---

## 6. Units

- **FEP position units = DDD world units** times the instance matrix's scale: `Sc = s_e x node
  row lengths` and `el+0x30`. For Sora's events `s_e = 1`, `el+0x30 = 1`; the node's row lengths
  are the bone's world scale (cases A, C) or 1 (cases B, D). **INFERRED** that Sora's bones carry
  no world scale (row length 1; the KH1 = DDD x 100 rule comes from bind poses). Check: read
  `|row|` of `[[Sora+0x120]+0x10]+0x38` with `tools/peek.py` while the game runs (not done; the
  game was not launched).
- **EAD offset**: raw s16 / 100 = DDD units, so the raw value is KH1 units.
- **Angles**: EAD radians x 1e-4; element `+0x20` and the FEP Rot commands degrees (README 8.1).
- **Example, the dive's ground ring** (P_FA_AJC_SHO0_0 part 12, MDL 9803AD0B, outer radius 16384
  raw = 1.0 (README 9.4, INFERRED), Scl 96 %): 0.96 DDD units = **96 KH1 units** radius in its
  own part frame; if the parent emitter part 16's scale (Sclac 337.83 % in x/z at its start)
  carries into its Generateanchor children, 3.24 DDD = **324 KH1 units** (not decoded: child
  placement, README 13.10). Centre: 1.30 DDD (130 KH1) ahead of Root at frame 32, on Root's
  plane.

---

## 7. Recipe for the KH1 side (per EAD-started element on Sora, each frame)

In DDD terms, column form, `W_b` the DDD bone's world matrix this frame (rotation `R_b` with any
scale, translation `t_b`), `R_e = Rz(rz)·Ry(ry)·Rx(rx)` (column form of row `RX·RY·RZ`: x first),
`off`, `s_e` from the event:

1. Node: follow (bit 2) -> `N = W_b(now) · T(off) · R_e`; stay -> `N = W_b(t0) · T(off) · R_e`
   frozen at the start (bit 3 set), or `N = T(t_b(t0) + off) · R_e` (bit 3 clear).
2. `P` = N's translation. `R_node` = N's rotation with each column normalised, `Sc = s_e ⊙`
   those column lengths; but bit 3 clear -> `R_node = R_e`, `Sc = s_e`; bit 5 -> `Sc = s_e`.
3. `A = T(P + el.off ⊙ Sc) · R_node · Rx·Ry·Rz(el.rot, degrees) · diag(el.scl ⊙ Sc)`
   (Flowmotion: `A = T(P) · R_node · diag(el.scl ⊙ Sc)`), handed to the interpreter as the
   instance matrix every update.
4. KH1: DDD units -> KH1 units is x100 joint for joint, so use `P x 100` (in KH1's world via
   `pose.rs`'s `to_world`) and `diag(100 · el.scl ⊙ Sc)`; the EAD offset's raw s16 already is KH1
   units. The FEP's own y/z negation stays inside the interpreter (kh1-renderer.md 6.3).
5. Time: `dt_fep = 0.5 x frame_step x game speed` per game frame (29.97 FEP frames/s); start late
   by `int(0.5 x (motion_t - t0))` frames when the trigger is late; handle fades in ticks.
6. Stop: as section 5 (dive/slam: never; Keyblade trail: stop request at t1, 8-tick fade on a
   motion change).

## 8. Open points

1. Whether the slam's 322 slow-down (frames 4.5-8.5) is the global game speed (FEP slows) or the
   motion's own clock (FEP does not): find what sets the motion speed in 322 / calls `0x53edf0`
   during Flowmotion (4.2).
2. Sora's bone world scale (6): assumed 1.
3. The node world update `0x5ce9a0` (modes `+0x70 & 0x200` etc.) and its call site per frame:
   assumed `local · parent world`, refreshed before `0x554ed0`.
4. Parent-to-child scale for Generateanchor children (the ring size, 6) and MDL 16384 = 1.0.
5. How the instance colour `+0x40` (EAD colour x alpha) reaches the draws.
6. `0x5af5d0` (the late-start kill bound) assumed to be the body length.
