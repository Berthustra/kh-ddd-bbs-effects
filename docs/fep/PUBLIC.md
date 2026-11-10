# DDD's FEP effect format: a write-up

This is a description of the visual effect format of *Kingdom Hearts 3D: Dream Drop Distance*
as shipped on PC in *Kingdom Hearts HD 2.8 Final Chapter Prologue* (Steam): the `.fep` effect
files, their `.ese` sound lists, the shared `resident.frr` resource pack, and the motion events
(`.ead`) that start effects. It covers the file layout, how the engine interprets an effect
frame by frame, the command classes, the resources, how they are drawn, and how an effect is
placed on a character and stopped.

It was written while porting DDD's Flowmotion effects to *Kingdom Hearts Final Mix* (the
kh1-flowmotion mod), and a working player built on it draws all 19 Flowmotion effects in KH1.
It is shared so other Kingdom Hearts modders don't have to start from zero.

**Nothing from the game is reproduced here**: no game data, no code, no disassembly listings,
no textures. The document gives offsets, field meanings, formulas in our own notation, and
addresses (RVAs) as pointers for anyone who wants to check a claim in their own copy of the
game.

## How to read this

- **Build.** `KINGDOM HEARTS Dream Drop Distance.exe`, Steam, inside KH 2.8 (PE timestamp
  `0x669E3C6E`). Every address is an **RVA** from image base `0x140000000`. Other builds will
  differ.
- **Method.** Static reading of the exe and the files only; DDD was never run under a debugger
  for this. Claims were checked by re-reading the code and by parsers run over the files.
- **Labels.**
  - **Unmarked** statements are **checked**: read in the exe at the RVA given, and/or verified
    on the data. "All 667" means verified on every FEP in the game. "Flowmotion" means the seven
    files the Flowmotion effects live in.
  - **INFERRED** follows from checked facts, but the link itself was not seen.
  - **GUESS** is a guess.
- **Detail.** This is a summary. The topic documents next to it hold the full tables, every RVA
  and the reasoning (see [Further reading](#10-further-reading)). Where they disagree with this
  file, this file and [README.md](README.md) section 14 hold the corrected version.

The engine's own namespace is `FunnyEffectsForPSP` (from RTTI). The models use PSP GE vertex
and primitive types, and the textures are 3DS (PICA200) data. The PC build reads these files as
they are.

## 1. Overview

```
FEP file
 ├─ names (section 7) ──> element id
 ├─ groups (section 0) ─> element ──> body (section 1)
 │                                     ├─ spawn events: at frame f, start parts [..]
 │                                     └─ parts
 │                                         ├─ slots: one command each (class id 1..157)
 │                                         ├─ init list + key lists: parameter blocks over time
 │                                         └─ 8 slot lists: which slots run in which phase
 └─ resources: TEX (2), MDL (3), ANM (4), VTL (5); or ids into resident.frr
```

- An **element** is the unit the game starts by name (e.g. `P_FA_AJC_SHO0_0`). It has one
  **body**: a length in frames, loop/end flags, and a list of **parts**.
- A **part** is a small program: a set of **slots**, each an instance of a **command class**
  (position, rotation, colour, matrix, generator, draw...). A part has a lifetime. It is started
  by the body's spawn events, or as a child by a generator command of another part. There is no
  separate particle record: a "particle" is a child part.
- Each slot reads a **parameter block**. The init list gives the first block; **key lists**
  swap in new blocks at given frames ("keys").
- Each frame, the interpreter calls the classes' methods (init, key, update, draw, release,
  loop rewind...) through a 9-entry vtable per class, in a fixed order of phases.
- Effects are started by **EAD events** on a character's motions. The event names the file,
  the element, a bone, an offset, and start and end times.

## 2. Container

### 2.1 Loading

FEP is resource type 10 (extension table `0x9ea180`); ESE is type 13. The loader `0x5ab930`
does the following:

- It checks the bytes `FEP` (not the 4th byte) and the version words `+4 == 1` and `+6 == 9`.
- It relocates offsets (`0x5ab330` → `0x5aadd0` per body).
- It registers the resources of sections 2, 4, 3 and 5, in that order.

**Offsets.** In the file every u32 offset counts from the **file start**, and 0 means "none".
After loading, offsets under section 1 become relative to the struct that holds them. Entries
of an offset table become relative to the struct that owns the table, not to the table itself.
A tool that reads the file directly can use file-relative offsets everywhere.

### 2.2 Header (0x48 bytes)

| off | type | meaning |
|---|---|---|
| 0x00 | char[3], u8 | `FEP` plus a byte that is not checked (0..65) |
| 0x04 / 0x06 | s16, s16 | 1, 9 (version; both must match) |
| 0x08 | u32 | offset of the command signature table (always 0x48) |
| 0x0c | u32 | 0 in the file; runtime flags |
| 0x10 | u32 | **id base** (bits 0..16 zero); top 4 bits = category |
| 0x14 | u32 | file size |
| 0x18 | u16 | category (= id base >> 28) |
| 0x1c..0x26 | u16 ×6 | counts of sections 0..5 |
| 0x28..0x44 | u32 ×8 | offsets of sections 0..7 |

Sections:

| section | contents |
|---|---|
| 0 | groups |
| 1 | bodies |
| 2 | TEX |
| 3 | MDL |
| 4 | ANM |
| 5 | VTL |
| 6 | special elements (empty in all 667 files) |
| 7 | names |

Sections 0..5 are u32 offset tables. Section 7 is `{s32 count, u32 → 12-byte records}`.

### 2.3 Command signature table

At 0x48 the file holds `{u32 158, u32 offset}`, then 158 records of 8 bytes:
`u8 a, u8 b, u16 c, u16 param_size, u16 work_size (in 2-byte units)`.

- The table is byte-identical to the exe's own copy at `0x9e7d10` in all 667 files.
- Element creation fails if any command id a body uses has a record that differs from the
  exe's (`0x5ac2f2`). This acts as a version check.
- No code reads `param_size` or `work_size`. Their meaning is confirmed by the data: blocks of
  those sizes tile the files without overlap.

### 2.4 Names, groups, elements

- **Element id** = `id base | group << 10 | element`. Bits 0..8 are the element, bit 9 marks a
  special element, bits 10..16 the group, and bits 28..31 the category.
- **Name record**, 12 bytes: `u32 element id, u32 length, u32 → NUL-terminated name`. Lookup
  is `0x5abb70`.
- **Group record**, 12 bytes: `u32 group id, u16 body index, u16 element count, u32 → element
  offsets`. In all 667 files, each group has exactly one element and each body one group.
- **Element**, 0x9c bytes. The useful fields:

  | off | meaning |
  |---|---|
  | `+0x10` | vec4 offset |
  | `+0x20` | rotation in degrees |
  | `+0x30` | scale (all three are applied by the instance matrix, §7.3) |
  | `+0x06` | view-angle test mode |
  | `+0x0a` | view-bounds test mode |
  | `+0x0b` | distance test mode |
  | `+0x90` | u16 start delay in frames |

  Several fields are still not understood (`+0x04`, `+0x08`, `+0x0c`, `+0x92`).

### 2.5 Body (0x34 bytes)

| off | meaning |
|---|---|
| 0x00 | flags. bit 1 = looping effect; bit 0 = end when no parts are left; neither = end at the length |
| 0x04 | length in frames |
| 0x0c | **stop mode** for a stop request: 0 = fade over 30 frames, 1 = fade over 15, 3 = ignore the fade and let the parts stop looping and die naturally, other = remove at once (`0x5b43a7`) |
| 0x10..0x1a | counts: spawn events, parts, command ids, 12, part groups, unused |
| 0x1c | → spawn events `{s16 frame, s16 n, s32 → u16 part ref[n]}`, in ascending frame order |
| 0x20 | → part offsets; an entry may be 0 (no part) |
| 0x24 | → u16 list of every command id the body uses (checked at creation) |
| 0x28 / 0x2c | 12 relocated records and "part groups": both present, never read (INFERRED authoring data) |

A **part ref** uses bits 0..11 as the part index. Bit 13 sets parent/child state bits (meaning
unknown).

### 2.6 Part (0x48 bytes)

| off | meaning |
|---|---|
| 0x00 | flags. bit 0 = stay alive while it has children; bit 1 = loop between `+0x0c` and `+0x0e` when the body loops |
| 0x02 | work-area size / 2 |
| 0x04 | lifetime in frames |
| 0x08 | slot count (0 = never spawned) |
| 0x09 | pass select. It is 2 in every part of every file, so only pass 1 is used |
| 0x0b | matrix type |
| 0x0c / 0x0e | loop start / end frame |
| 0x13 | key-list count |
| 0x14..0x1b | counts of the 8 slot lists |
| 0x1c | → slot descriptors |
| 0x20 | → init list |
| 0x24 | → key lists |
| 0x28..0x44 | → the 8 slot lists (u16 slot indices) |

- **Slot descriptor:** `u16 command id, u16 work offset / 2, u16 n, u16 0, s32 → {u16 slot[n],
  u16 work[n]}`. The second part lists the **inputs**: earlier slots whose output (the first vec4
  of their work) this command reads. For example, the matrix command reads a position, a scale
  and a rotation slot.
- **Init and key lists** share one layout: `{s16 frame, s16 smallest span, s32 count, s32 →
  entries}`. Each entry is `u16 slot, u16 parameter offset (from the list header), s16 start
  frame, s16 span`. The init list has one entry per slot. The first key list is at frame 0, but
  it does not always restate every slot.
- **Slot lists:** list k holds exactly the slots whose class overrides vtable method k. This is
  true of every list in every file, with one exception: the draw list may be empty even when a
  drawing class is present (361 parts game-wide). A player must use the file's lists.

## 3. The interpreter

### 3.1 Phases (vtable slots)

Each class is a static object with a 9-method vtable (class table `0x9e7450 + id*8`):

| vtable slot | phase | dispatcher |
|---|---|---|
| 0 | init at spawn (returning 0 aborts the spawn) | `0x5b56f0` |
| 1 | key: a new parameter block takes over | `0x5b5180` |
| 2 | update, every frame (list `+0x28`) | `0x5b5f10` |
| 3 | draw (list `+0x2c`) | `0x5b59d0`, `0x5b5c20` |
| 4 | release at part death (list `+0x30`) | `0x5b82b0` |
| 5 | loop rewind (list `+0x34`) | `0x5b6680` |
| 6 | first update only (list `+0x3c`) | `0x5b5f10` |
| 7 | after all updates (list `+0x40`) | `0x5b5f10` |
| 8 | list `+0x44`; overridden only by Wave_tail / Wave_bindlocus; its caller is unknown | n/a |

### 3.2 One frame

1. **Time** (`0x554f80` → `0x5b4290`). The effect time and every part's time advance by dt.
   Then, in order:
   - parts past their lifetime die (their release list runs);
   - spawn events whose frame has been reached spawn their parts, with the start time carried
     over to the fraction;
   - the body end is checked (§2.5).
2. **Update** (pass 1, `0x5b4700`). This runs per instance after the instance matrix is built
   (§7.3), in this order:
   1. first-update slots;
   2. due key lists, in entry order;
   3. update slots;
   4. after-update slots.
3. **Draw.** The draw list runs in two passes. The second holds only FecLensflare_light.

**Time base.** The game's tick is 1/59.94 s, scaled by the global game speed. The FEP clock
advances **0.5 per tick**, so **one FEP frame = 1/29.97 s**. Per-motion speed changes don't
reach the FEP clock: they only delay the triggers.

### 3.3 Keys, loops, spawning

- **Key** (`0x5b5180`): while the next key list's frame ≤ the part time, each entry runs its
  slot's key method with (new block, old block, old span). The order of entries matters.
- **The ac forms re-base at a key** (`0x5afe70`). Take `t` = the old span. Then:
  - new base = old position at `t` − old parameter + new parameter;
  - the same rule applies to velocity;
  - new acceleration = acceleration − old acceleration + new acceleration.

  So random offsets, the spawn velocity and gravity carry across keys.
- **Loop rewind** (`0x5b6680`): for each slot in the loop list, the key active at the loop
  start is found **by class id** (not by slot). If no key matches, the init list is used. Two
  slots of the same class therefore get the same key: an exact replay has to copy this quirk.
- **Spawn** (`0x5b73d0`): allocates a part instance, runs the init list and links the part in.
  A child spawned during the update has its due keys run immediately.

### 3.4 Runtime state

There is no fixed particle struct. Each command keeps its state in its own slice of the part's
work area, and its output is the first vec4 there. Children receive a **spawn info** block:
flags plus pointers to position, velocity, acceleration, rotation, scale and colour. The
Pos/Rot/Scl/Col commands take these values at init. FecPosac adds them; the others replace.

## 4. Command classes

158 ids, of which 157 are real classes (names from RTTI). The 19 Flowmotion effects use 31 of
them, plus FecFlash in the Flowmotion aura. Behaviour is **checked** unless marked.

| ids | classes | what they do |
|---|---|---|
| 1-3 | FecPos, Posmv, Posac | position. Plain value; value + v·t; or value + v·t + a·t²/2 |
| 4-6 | FecRot, Rotmv, Rotac | rotation in **degrees** (same three forms) |
| 7-9 | FecScl, Sclmv, Sclac | scale in **percent** (output × 0.01) |
| 15-17 | FecCol, Colmv, Colac | RGBA colour, clamped to 0..255. **128 = 1.0** everywhere it is used |
| 14 | FecLmatrix | builds the part's local matrix from its Pos, Scl and Rot inputs: S·R·T, with **y and z of position and rotation negated** first. Param byte 3 = rotation order (ZYX, ZXY, YZX, YXZ, XZY, XYZ). Byte 1 & 7 = orientation for drawing (0 own axes, 1 screen billboard, 2 billboard about the frame's Y). Byte 4 = sort group |
| 18 | FecGenerate | continuous emission: rate per frame, interval, chance %, random cube, speed, acceleration, child part |
| 151 | FecGenerateanchor | one burst of N children per key that names it (and per loop) |
| 150 | FecGenerateanchorvertex | children placed on a model's vertices, in VTL index-list order |
| 19 | FecGravity | adds g/100 × 0.098 to the input Posac's y acceleration at a key |
| 22-25 | FecRndpos, Rndrot, Rndscl, Rndcol | at a key, perturb the input's value, velocity or acceleration by a random amount in [−r, r], [0, r] or [−r, 0]. Skipped once the part has looped |
| 61, 62 | FecColorblink_parameter, Colorblink | a blink factor and its application to a colour |
| 68, 79, 80 | Locus_tail_strategy, Null_routegenerator, Shapetail | a trail: the part's past positions resampled at fixed arc spacing, one ANM sprite per point, colour and size lerped from head to tail |
| 105, 106 | Bindlocus_polygon, Wave_bindlocus_strategy | a ribbon: two tracked points' histories (each older sample drifting by its own velocity plus a random "wave") drawn as a textured strip, optionally B-spline smoothed, with UV scroll. The Keyblade trail is one of these |
| 100, 104 | Radialg_line_particle, Ellipse_line_particle | line-strip particles: shot in a cone with gravity, or spinning on an ellipsoid |
| 152, 153 | FecAnm_plus, Megaanm_plus | ANM flip-book sprites: one per part, or a ring buffer of N sprites inside one part, each with its own clock |
| 154, 155 | FecMdl_plus, Mdlts_plus | MDL models; Mdlts adds a UV scroll |
| 107 | FecFlash | a full-screen (or rectangle) flash. The flash as Flowmotion begins is one |

- **`_plus` classes** (152..155) have all-zero signature records. Their parameter blocks are the
  base class's layout plus at least one trailing byte. For the sprite and model classes this
  byte is the **blend selector** (§6.3).
- **Randomness.** Every effect random draw goes through one global **SFMT19937** generator
  (`0x613020` → `0x6196a0`). The unit draw is `(r >> 1) × 2⁻³¹`, which is in [0, 1). Exact
  replay of DDD's random sequence is therefore possible in principle, but it is shared with the
  rest of the game.
- Generators pass the spawn flags 7 (position, velocity and acceleration) to their children.
  Spawn events pass none.

Per-class parameter layouts and exact formulas: [commands.md](commands.md) (transforms,
Lmatrix), [generators.md](generators.md) (generators, Rnd*, Colorblink, RNG),
[tails.md](tails.md) (trails, ribbons, line particles), [draw-sprites.md](draw-sprites.md),
[draw-models.md](draw-models.md), [aura.md](aura.md) §5 (FecFlash).

## 5. Resources

### 5.1 Records and ids

Sections 2..5 hold 24-byte records `{f32 scale, u32 id, u32 size, u32 offset, i32 −1, u32 0}`.
A record with id 0 is skipped. Each id is one of two kinds:

- **Local:** `0x80000000 | ... | type << 7 | index`. The type is 0 TEX, 1 ANM, 2 MDL or 3 VTL.
  The resource is loaded from the FEP itself.
- **Resident:** `0xF0000000 | (k >> 7) << 26 | type << 7 | (k & 0x7f)`. The resource is entry
  k of `effect/resident/bin/resident.frr`'s table for that type.

`resident.frr` starts with `FRR\0` and holds 4 tables (32 TEX, 134 ANM, 136 MDL, 24 VTL). Its
resource blocks are the same as in an FEP.

### 5.2 TEX / CTRT textures

The texture data is 3DS (PICA200) data, in formats 0..13. The pixels are 8×8 tiles, row-major
from the top, with Morton order inside each tile. There is no vertical flip.

- **ETC1 and ETC1A4 base colours are not bit-replicated** by the game's decoder (`c4 << 4`,
  `c5 << 3`, at `0x671c67`). A standard ETC1 decoder differs by up to 14 levels. Decoding the
  game's way matches the 1× remastered DDS files bit for bit.
- **HD replacement.** The PC build replaces textures by ordinal from
  `remastered/effect/.../<file>/-<n>.dds` (`0x678430`). For Flowmotion, 9 of the 18 textures are
  replaced with different (2× or 4×) art in the same UV layout. To look like the PC game, use
  the DDS files.

### 5.3 MDL models

- **Header:** `MDL\0`, version 1/10. `+0x10/+0x14` is the texture-reference list (24-byte
  entries with the texture id and blend code). `+0x18/+0x1c` gives the meshes.
- **Tree:** mesh (0x44 bytes) → sub (16 bytes) → group (16 bytes).
- Each **sub** carries a **PSP GE vertex type**, and each **group** a **PSP GE primitive**
  (triangle strip or triangles).

There are four vertex types (traced in the exe, `0x5be1d0`):

| vtype | stride | layout |
|---|---|---|
| `0x11e` | 16 | s16 u, v; u32 RGBA8; s16 x, y, z |
| `0x11c` | 12 | u32 RGBA8; s16 x, y, z |
| `0x102` | 10 | s16 u, v; s16 x, y, z |
| `0x100` | 6 | s16 x, y, z |

Scales: position **s16 / 32767**, UV **s16 / 16384**. Vertex colour **0x80 = 1.0**. A vertex type
without colour acts as 0x80.

### 5.4 ANM 2D animations

ANM is a flip-book (CAnm2D::Load `0x55a5f0`) with these lists:

- texture list;
- keys `{flags, frame, duration}`;
- frames `{flags, n, → quad indices}`;
- quads (4 vertices, a blend code).

The quad vertices are 10 bytes, `{u16 u, u16 v, s16 x, s16 y, s16 z}`, in the 0x102 layout,
with **u, v / 16384** (INFERRED from the data). The two top corners come first, then the two
bottom ones (a strip).

### 5.5 VTL and ESE

- **VTL** holds u16 index lists. Its loader is a no-op. FecGenerateanchorvertex reads them as
  vertex orderings (INFERRED).
- **ESE** is the sound list of an FEP: `ESE\0`, an entry count, entry offsets. An entry is
  `{u16 sound count, s16 name index, char[16] element name, n × {char[32] sound name, u32}}`. It
  is matched to the FEP's elements **by name** on first use, not by position.

## 6. Drawing

DDD on PC draws through **Direct3D 12**. A GL-like layer on top is fed by 3DS-style state.
Details: [render-state.md](render-state.md), [draw-models.md](draw-models.md),
[draw-sprites.md](draw-sprites.md).

### 6.1 Models (FecMdl_plus `0x58a110`, FecMdlts_plus `0x588f90`)

The model matrix, in row-vector form (`v' = v·M`):

`M = S · L · F`

- S = uniform scale: the record's f32 × mesh 0's `+0x24`.
- L = the part's Lmatrix output. It is turned toward the camera first when Lmatrix byte 1 & 7
  says so.
- F = the frame: the owner part's frame matrix, or the instance matrix (§7.3).

UV: `uv = (s16/16384) × (uvscale/2) + uvoff + scroll/texsize`. Mdlts_plus adds a **scroll**,
which is `p + v·t + a·t²/2` in texels, over the part's time. Texture addressing wraps, so the
scroll repeats.

### 6.2 Sprites (FecAnm_plus `0x586f10`, FecMegaanm_plus `0x58e010`)

Both draw one ANM frame per sprite through one shared core (`0x55a360` → `0x559eb0`). The core
multiplies each quad's colour by the command colour as `(c × C) >> 7`.

- **Anm_plus** orients the quad by Lmatrix byte 1 & 7 (own plane, screen billboard, or
  billboard about the frame's Y axis).
- **Megaanm_plus** sprites are always screen-aligned, each with its own rotation, scale, colour
  and fade.

### 6.3 Shading and blend

Colour formula (vertex shader; checked against the shader constants):

```
colour = tex.rgb × vcol.rgb × K.rgb        (vcol: 0x80 = 1.0; K: 0x80 = 1.0, so up to ×2)
alpha  = tex.a   × vcol.a   × K.a
discard if alpha ≤ 0.5/255
```

K = the command's colour input (Col/Colac) × the instance colour (which carries the stop fade).

| blend index | from | result |
|---|---|---|
| 0 | n/a | opaque |
| 1 | selector 0, 3, 5-7; code `0x70600` | normal alpha blend (SRC_ALPHA, INV_SRC_ALPHA) |
| **2** | selector 1; code `0xA0600` | **additive, weighted by alpha** (SRC_ALPHA, ONE) |
| 3 | selector 2; code `0xA0602` | subtractive (REV_SUBTRACT, SRC_ALPHA, ONE) |

The shock-wave models of the dive and the slam, and all 394 Flowmotion ANM quads, use index 2.
The other render state:

- depth test LESS_EQUAL, depth write off;
- no culling;
- sampler: wrap addressing, linear filtering (4× anisotropic).

With additive blending and no depth write, the draw order does not change the result.

### 6.4 Sort groups

Lmatrix (`0x5bc450`) links every drawn part into a bucket, and buckets are drawn in groups
0..8. Only groups 7 and 8 change render state (depth test off). Which group an EAD-started
effect lands in by default is **open** (GUESS: not 7/8, since these are world effects hidden
behind walls).

## 7. Placement and triggers

Details: [space-time.md](space-time.md), [triggers.md](triggers.md), [aura.md](aura.md).

### 7.1 The EAD event (52 bytes)

Each motion of a character has EAD events (`p_ex01_000.ead` for Sora, `w_so01_001.ead` for his
Keyblade). The event fields:

| off | meaning |
|---|---|
| 0x00 / 0x04 / 0x08 | → effect file name, element name, bone name (a missing bone means the world origin) |
| 0x0c | s16 ×3 offset in the bone's frame, × 0.01 = DDD units |
| 0x12 | s16 **effect group** filter |
| 0x18 | s16 ×3 rotation, × 0.0001 = radians, applied X, then Y, then Z |
| 0x1e | s16 ×3 scale, × 0.01 |
| 0x24 | RGBA colour, × 1/128 |
| 0x28 / 0x2c | start / end time (end 0 = none), in motion time units of 1/60 s |
| 0x2a | low 12 bits = fade ticks on a motion change |
| 0x2e / 0x2f | fade-out / fade-in ticks |
| 0x30 | flags (below) |
| 0x33 | max starts per motion lap |

The flags at `+0x30`:

- bits 0-1: what a motion change does (fade, stop request now, **keep**, or stop request later);
- bit 2: follow the bone;
- bit 3: take the bone's rotation;
- bit 4: send a stop request at the end time (rather than fading out);
- bits 12-15: a level filter.

### 7.2 Triggers

**Every Flowmotion effect is started by an EAD event; none is started by name from code.**

The code decides which of a motion's events run by passing an **effect group** with the motion.
An event runs when its group is negative, when the passed group is negative, or when the two
are equal (`0x554070`). For Flowmotion, the group comes from the command being executed (its
attack-data record `+4`). Only one case overrides it in code: the rail's boss variant.

**The Flowmotion aura.** On every fresh entry into the Flowmotion state, `0x2bcb60(player, 200,
0)` starts **every** event of the pseudo-motion `001` with group 200, all at once, on Sora and
on his Keyblade. That is 18 + 1 events: glows on the chest, limbs, head and pelvis, plus a
screen flash. These events use the "keep" mode, so motion changes don't affect them. Leaving
Flowmotion through the common back-to-normal routine `0x2a7b80` sends them a **stop request**
(no kill).

### 7.3 The instance matrix

At start (`0x5521e0`), an effect node is made. Depending on flag bits 2 and 3, it is either
parented to the bone or placed once:

| case | flags | node | moves? |
|---|---|---|---|
| A | follow + bone rotation | `[R_e │ off] · W_bone(now)` | yes, fully |
| B | follow, no bone rotation | position from the bone, axes from the event | position only |
| C | stay + bone rotation | as A, frozen at the start | no |
| D | stay, no bone rotation | bone position at the start, event axes | no |

Every update, `0x5ae260` builds the instance matrix from the node (position `P`, basis `E`,
scale `Sc`) and the element fields:

```
M3        = diag(el.scale ⊙ Sc) · RZ·RY·RX(el.rot) · E
row 3     = el.offset ⊙ Sc + P
```

`el.offset` is in world axes, scaled but not rotated.

**Units.** FEP positions are in DDD world units (× the instance scale). For a KH1 port, KH1
Sora's bind pose is DDD's × 100.

### 7.4 What stops an effect

- **Its own length.** The body flags decide (§2.5). Non-looping effects simply play out, even
  after the motion changes.
- **The event's end time.** Either a stop request, which is then handled by the body's stop
  mode, or an alpha fade over `+0x2e` ticks.
- **A motion change.** For looping effects the event's mode decides: fade, stop now, keep, or
  stop later. Non-looping effects are left alone unless flags bits 6-7 equal 1.
- **The handle's alpha.** Fades multiply the instance colour's alpha, and alpha 0 hides the
  instance.

## 8. Checked on data

For example:

- The parser covers every byte of all 667 FEPs: no crashes, no problems, no uncovered non-zero
  bytes.
- The signature table is identical in all 667.
- All 267 Flowmotion draw indices name an existing resource with the right magic.
- All 16 resident textures that have a 1× DDS decode bit-identically to it.
- The shock-wave models render as rings, domes and cylinders consistent with the game.
- A player built on this document draws all 19 Flowmotion elements and the aura. The mod's
  author compared it with DDD in game and judged it to match.

## 9. Open questions

- The remaining fields listed as not understood in §2 (signature bytes a, b, c; element `+0x04`,
  `+0x08`, `+0x0c`, `+0x92`; body `+0x02`, `+0x08`, `+0x0b`; part `+0x10..+0x12`; the `_plus`
  trailing fields of the model and sprite classes beyond the blend selector).
- Who calls vtable slot 8 (Wave_tail_strategy, Wave_bindlocus_strategy).
- Shapetail's orient mode (`+0x47`, one Flowmotion use).
- Lmatrix orientation modes 3, 4 and bit 3, and the non-Flowmotion matrix types.
- Which sort group an EAD-started effect gets, and whether scene fog applies to effects.
- The exact semantics of the category mask: two code paths test it differently (`0x5b4311`
  value AND, `0x5b4717` bit index).
- Classes not used by Flowmotion: most of the 157 were not decoded. The container,
  interpreter and resource parts apply to all of them.

## 10. Further reading

The detailed documents next to this one:

| doc | contents |
|---|---|
| [README.md](README.md) | consolidated spec: container, interpreter, the Flowmotion elements class by class, unknowns, corrections |
| [structure.md](structure.md) | container, header, sections, bodies, parts, relocation (first decode) |
| [commands.md](commands.md) | interpreter, runtime structs, transform and matrix classes |
| [generators.md](generators.md) | generators, the RNG, Rnd*, Colorblink, Gravity |
| [tails.md](tails.md) | trails, ribbons, line particles, the Keyblade trail |
| [draw-models.md](draw-models.md) | model draw: mesh reader, vertex formats, matrix, colour, UV scroll |
| [draw-sprites.md](draw-sprites.md) | ANM sprites and the sprite ring buffer |
| [render-state.md](render-state.md) | D3D12 blend, depth, cull, sampler, shaders, sort |
| [textures.md](textures.md) | TEX/MDL/ANM/VTL, resident.frr, texture decoding |
| [space-time.md](space-time.md) | EAD events, placement, clocks, stop rules, units |
| [triggers.md](triggers.md) | what starts each Flowmotion effect |
| [aura.md](aura.md) | the Flowmotion aura and FecFlash |
| [kh1-renderer.md](kh1-renderer.md) | design notes for drawing these effects in KH1 |

A reference implementation is the `fep` crate in this repository (`fep/`, Rust, MIT OR
Apache-2.0). It parses FEP and `resident.frr` and runs elements as described here; drawing is
left to the caller. The mod's `src/fx/` draws them in KH1.

These notes and the crate are licensed MIT OR Apache-2.0, like the rest of the repository.
Kingdom Hearts is a trademark of Disney and Square Enix; this work is not affiliated with or
endorsed by them.
