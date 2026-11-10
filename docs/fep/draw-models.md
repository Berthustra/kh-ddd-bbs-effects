# Drawing models: FecMdl_plus (154) and FecMdlts_plus (155)

How DDD PC draws an MDL model for the two model draw commands: the mesh reader, the vertex
formats and their scales, the model matrix, the colour, the UV transform and scroll. The GPU side
(combiner/shader formula, blend, depth, cull, sampler, draw order) is in `render-state.md`.

DDD Steam exe (KH 2.8), image base 0x140000000, every address an RVA. Labels as in README.md:
unmarked = read in the exe at the RVA given and/or checked on the data of the Flowmotion FEPs;
**INFERRED** = follows from read facts; **GUESS** = not read. Tools: `ddd_fep_models_fep.py`
(MDL reader and parameter dump), `ddd_fep_models_render.py` (renders the shock models to PNG,
section 9), `ddd_fep_models_disp.py`, `ddd_fep_models_emu.py`.

## 1. Call chain

| step | RVA | what |
|---|---|---|
| FecMdlts_plus draw slot | `0x588f90` (same body as FecMdlts `0x5882a0`) | builds matrix, colour, UV scroll; calls `0x5be8c0` at `0x5896f6` |
| FecMdl_plus draw slot | `0x58a110` (same body as FecMdl `0x5899a0`) | same without scroll; calls `0x5bdfb0` |
| CMdl draw (with scroll) | `0x5be8c0` -> `0x5be440` | per mesh, per sub: state, texture, TexScale, then `0x5be1d0` |
| CMdl draw (no scroll) | `0x5bdfb0` -> `0x5bdb70` | identical loop, scroll = 0 |
| sub draw | `0x5be1d0` | vertex format from the sub's vertex type, one draw per group |
| GL-like layer | `0x6007c0` (vertex pointer), `0x5e33f0` -> `0x66ff70` -> `0x671980` -> `0x670db0` | picks the vertex emitter and shader program, emits the vertices |

The only difference between `_plus` and the base classes is the blend selector byte (section 6).

## 2. MDL structures as the draw reads them

Read by `0x5be440` (`0x5be47d..0x5be7cb`), `0x5be1d0`, `0x5bee20`, `0x5bee50`. Offsets from the MDL
start (all relocated at load, README 9.4).

- MDL `+0x14` -> texref table (24 B entries), `+0x18` mesh count, `+0x1c` -> meshes (0x44 B).
- **Mesh** (loop `0x5be5c6..0x5be7cb`, `r13 = mesh + 0x1c`):
  - `+0x1c` sub count, `+0x20` -> subs (16 B);
  - `+0x28`, `+0x2c` f32 **UV scale** (u, v), each **x 0.5** (`0x5be6fe`, `0x5be702`);
  - `+0x30`, `+0x34` f32 **UV offset** (u, v) (`0x5be6ed`, `0x5be6f3`);
  - `+0x24` f32 **model scale**, read from **mesh 0 only** (`0x5bee50`, called at `0x589090`);
  - `+0x38..+0x40` f32 vec3 **pivot**, mesh 0 only (`0x5bee20`), used when the section-3 record's
    flags bit 0 is set (section 4);
  - **not read by the draw**: `+0x04`, `+0x0c`, `+0x10`, `+0x14`, `+0x18`. The "colour tables"
    (mesh `+0x0c`/`+0x14`, group `+0x04` when mesh `+0x10` > 1) play no part in this draw path
    (INFERRED: they serve another model player; FecMdl/FecMdlts never look at them).
- **Sub** (16 B; `0x5be5f0..0x5be7b3`, `0x5be1d0`): `+0x00` vertex type, `+0x04` texref index
  (-1 = untextured), `+0x08` group count, `+0x0c` -> groups.
- **Group** (16 B; `0x5be258..0x5be2a9`): `+0x00` primitive, `+0x08` vertex count, `+0x0c` ->
  vertices. `+0x04` is not read. No index buffer (index pointer 0, `0x66ff70`).
- **Texref** (24 B): `+0x04` texture-manager slot (written at load), `+0x08` flags (bit 2 = no
  depth write, bit 3 = cull; render-state.md 4, 5), `+0x0c` blend code (used only when the blend
  selector is 4). `+0x10`, `+0x14` not read.

Primitive (`0x5be258..0x5be26d`): group `+0` = 3 -> internal 5 = **triangle list**, 4 -> internal 6
= **triangle strip** (the emitter starts a new strip per group: its "kick" flag is 0x1000 on the
first two vertices only when the primitive is 6, `0x66a40a..0x66a416`); any other value -> 0
(nothing drawn, INFERRED). PSP GE numbering; only 3 and 4 occur (README 9.4).

## 3. Vertex formats (traced, not inferred)

`0x5be1d0` asks the vertex-format factory `[0xaaa950]` (class vtable `0x7f42a0`; init `0x600ef0`
sets mode 5 and the format ids) for the sub's vertex type (vtable `+0x10` = `0x6011c0`). Only 8
types exist; the four MDL types map to format ids 0xd..0x10 (`0x600f24..0x600f42`), and
`0x670db0` (jump table `0x67140c`) picks one emitter per id. Each emitter converts to a
28-byte vertex {f32 x, y, z; u32 RGBA8; f32 u, v; u16 kick}.

| vtype | stride | layout (byte offsets) | emitter | position | UV | colour |
|---|---|---|---|---|---|---|
| `0x11e` | 16 | s16 u @0, s16 v @2, u32 RGBA8 @4, s16 x,y,z @8 (pad 2) | `0x66a3e0` | s16 x (1/32767) | **signed** s16 x (1/16384) | from vertex |
| `0x11c` | 12 | u32 RGBA8 @0, s16 x,y,z @4 (pad 2) | `0x667f90` | s16 x (1/32767) | constant (0.5, 0.5) | from vertex |
| `0x102` | 10 | s16 u @0, s16 v @2, s16 x,y,z @4 | `0x66baf0` | s16 x (1/32767) | signed s16 x (1/16384) | 0xFFFFFFFF, but ignored: see render-state.md 3 |
| `0x100` | 6 | s16 x,y,z @0 | `0x6686a0` | s16 x (1/32767) | constant (0.5, 0.5) | 0xFFFFFFFF, ignored |

- Scales: position `[0x81b49c]` = 3.05185e-05 = **1/32767** (`0x66a44a`), UV `[0x81b4a0]` =
  6.10352e-05 = **1/16384** (`0x66a457`). This corrects README 9.4 ("16384 = 1.0" for position):
  a position of 16384 is **0.5** model units before the model scale.
- V is **not flipped** in mode 5 (`0x66a4ef..0x66a51e`: `1 - v` only when the mode is not 5; the
  factory sets mode 5). Texture row 0 = the first decoded row (textures.md), so a renderer samples
  the decoded/DDS image as-is with v = 0 at the top.
- Colour byte order: byte 0 = R, byte 3 = A (copied raw, `0x66a4e9`); the input layout reads it as
  UNORM, so 0x80 -> 0.502 in the shader, and the shader multiplies by 255/128 (render-state.md 3):
  **vertex colour 0x80 = 1.0**, 0xFF = 1.99.
- Vertex colours of the shock models (section 9): all use 0x00..0x80 (never above), e.g. discs
  9803AD0B (dive 12) and 9803990C (slam 2) ramp RGB and A from 0 (inner edge) to 0x80 (outer);
  others keep A = 0x80 and ramp only RGB, or (F000017C) keep RGB 0x80 and ramp A.

## 4. Model matrix

All row-vector (v' = v M). Built in `0x588f90..0x5896f6`:

1. **Scale** `s = record.f32 (+0x00 of the section-3 record, 0x588ff8..0x589089) x mesh0.f32 +0x24`
   (`0x589090..0x5890a7`) -> uniform scale matrix S (`0x5b1552..0x5b1571`, argument 7 of `0x5b1510`).
   Section-3 records hold 1.0 (0.95 in 3 records game-wide); mesh +0x24 is e.g. 1.0406 for 9803AD0B.
2. **Local** L = part `+0x78` (Lmatrix output, README 8.2). If Lmatrix byte 1 & 7 == 1 the 3x3 of L
   is first multiplied by the camera rotation (`[0xaa3e10 + byte1*8] + 0x160`, `0x58917b..0x5892c1`):
   a full billboard. Not used by any shock part (section 8).
3. **Frame** F = frame owner's `+0x88`, or the instance's `+0x60` when there is none
   (`0x5b152e..0x5b154e`). `0x5b1510` then branches on Lmatrix byte 1 & 7: 0 = plain
   (`0x5b157b..0x5b15be`: M = S L F, INFERRED order from `0x5b2de0` then `0x63ac90`), 1 = camera
   facing, 2 = upright, turned about Y towards the camera (`0x5b1dc9..`, atan2 `0x771c58` when bit 3
   is set), 3 / 4 = other camera modes (`0x5b1844`, `0x5b1a49`), not decoded.
4. **Pivot**: if the section-3 record's flags bit 0 is set, T -= pivot x M3x3 (pivot = mesh 0
   `+0x38`, `0x589380..0x589414`). All shock models have flags 0.

The final M goes to the shader's world matrix (`0x56af40` -> LocalProjection[0], render-state.md 3).

Model space -> FEP units: `p_fep = (s16 / 32767) x s x L x F`. The y/z negation of the Lmatrix
(README 8.2) is already inside L.

## 5. Colour

- **Constant colour K** (the "TEV constant" = shader `Ambient`): K = clamp(input 0 (Col/Colac
  output, 0..255) x element colour (`inst+0x40`, alpha = stop fade x user alpha), 0, 255) per
  channel (`0x589419..0x5894b8`), truncated to bytes (`cvttss2si`, `0x5be4b0..0x5be4df`), RGBA8;
  `0x80808080` when the draw has no colour (`0x5be4e8`). Passed to `0x5e29a0` (`0x5be734`).
- In the shader K is divided by **128** (combiner word `0x202`, `0x6711f8..0x671289`):
  **K = 0x80 means x1.0**, 0xFF means x1.99. Colac's 0..255 range therefore reaches about x2.
- Final fragment = texture x vertex colour x K (render-state.md 3).

## 6. Blend selector and per-sub state

- `_plus` trailing byte (Mdl_plus `+0x1a`, Mdlts_plus `+0x4b`) = selector: 4 -> the texref's blend
  code via `0x5c02a0` (0x70600 -> 1, 0xA0600 -> 2, 0xA0602 -> 3, else 0); otherwise table
  `0x7ed8e0[sel]` = {1, 2, 3, 1, (0), 1, 1, 1} (`0x5c02d0`). **All Flowmotion blocks use 1 -> blend
  index 2**, the same as their MDL code 0xA0600. Blend index -> D3D12 state: render-state.md 2.
- Per sub (`0x5be621..0x5be68d`): depth write = !(texref.flags bit 2), cull enable = texref.flags
  bit 3. All Flowmotion texrefs have flags 7 -> **no depth write, no culling**.
- Texture: the sub's texref slot, bound only when it changes (`0x5be71a..0x5be758`,
  `0x5c0710(mgr, cmd, slot, 5, 1, 0, 0)`). A sub with texref -1 switches to the untextured combiner
  preset (`0x5be762..0x5be782`).

## 7. UV transform and scroll

Per sub (`0x5be6a1..0x5be712`, `0x601a90` -> `0x6703c0` -> `0x65ce90(0, ...)` = shader TexScale):

    TexScale = (mesh.uvscale.u x 0.5, mesh.uvscale.v x 0.5,
                mesh.uvoff.u + scroll.u / tex.width, mesh.uvoff.v + scroll.v / tex.height)
    uv = vertex.uv(s16 / 16384) x TexScale.xy + TexScale.zw          (VS blob 23)

so `u = s16 / 32768 x uvscale.u + uvoff.u + scroll.u / width` (same for v with height; width/height
= TEX `+0x1c/+0x20`, `0x5c06c0`). Texture addressing is **wrap** (render-state.md 6), so the scroll
repeats. The VS multiplies by TPage (cbConstantVS +72), 1.0 for these textures (INFERRED: it is
1/size only in the rectangle-texture mode, `0x649b21`).

- **Scroll** (Mdlts_plus only): param `+0x10/+0x14` value p, `+0x18/+0x1c` velocity v,
  `+0x20/+0x24` acceleration a (copied to work by slot 0 `0x588f60`, rebased by slot 1 `0x588c80`
  with randoms from `+0x28..` when byte `+0x4a` is set): scroll = p + v t + a t^2/2 in **texels**
  (`0x5890ca..0x589125`), t = the part's time.
- Example: 9803AD0B (dive part 12) has uvscale (2.0, 1.0) -> u = s16/16384, v = s16/32768; scroll
  v = 5 t texels on a 64-high texture.

## 8. Other parameter fields, and the shock parts' Lmatrix bytes

Mdlts_plus block (`0x4c` bytes; names from `ddd_fep_models_fep.py`):

| offset | read at | meaning |
|---|---|---|
| `+0x0c` s32 | `0x588fbc`, `0x5892c6` | section-3 model index |
| `+0x10..+0x27` | slot 0/1 | scroll p, v, a (section 7) |
| `+0x28..+0x3f` | slot 1 | scroll randoms (used when `+0x4a`) |
| `+0x40` f32, `+0x48` byte | `0x5894be..0x5896b3` | **depth push**: when `+0x48` != 0, `0x5baa40` computes a view-depth shift from the model origin (camera `[0xaa3e48]`, near `[0xaa3e00]+0x1ac` x 1.1) and passes it to `0x5e3090` -> `0x66fe00` -> VS ParamVector[3] (render-state.md 7). INFERRED: pulls the model towards the camera, clamped at 1.1 x near |
| `+0x44` byte | not found in the draw | **GUESS** unused here (1 only in slam part 11) |
| `+0x45` byte | `0x5896a2` | 0 -> the draw resets the viewport/scissor to the full 3DS screen (`0x5e30f0`); all Flowmotion blocks 0 |
| `+0x49` byte | `0x58953a` | passed to `0x5b39a0` in the depth-push path (mode; 3 in all blocks) |
| `+0x4b` byte | `0x5896a7` | blend selector (section 6) |

Lmatrix bytes of the shock parts (`ddd_fep_models_fep.py params`): dive SHO0 parts 12-15 `03 00 ..`
(byte 1 = 0, plain frame); slam SHO0 parts 2, 9, 10 `01 00 ..` (plain); slam parts 7, 8 (the cups)
`01 02 ..`: **byte 1 = 2 = turned about Y to face the camera** (bit 3 clear, so without the
atan2 step; the rest of mode 2, `0x5b1e5c..0x5b2137`, re-normalises the frame against the camera
matrix: not fully decoded). The cup is round about Y, so this only turns its UV seam towards the
camera (INFERRED). Slam part 11 `04 00 ..` plain. Byte 1 low bits = 1 (full billboard) is used by
no shock part. `+0x48` (depth push) is 0 in all shock blocks (dive parts 4-8 and slam part 3 use it).

## 9. Check: the shock models rendered (`ddd_fep_models_render.py`)

Output: `%TEMP%/khfm/fepwork/models/png/` (`shock_models.png` + one PNG per model; `--t 8` writes
frame 8 to `png/t8/`). Each row: top view | 30-degree view | vertex alpha only. Rendered with the
1x decoded textures, K = 0x80, additive on black. All ten models are vtype 0x11e (16-byte vertices),
one mesh, one sub, texref flags 7, blend code 0xA0600; primitives 3 and 4 only.

| model (part) | scale s | shape in model units x s | vertex colour / alpha | look |
|---|---|---|---|---|
| 9803AD0B (dive 12) | 1.041 | flat disc y = 0, r 0.19..0.50 | RGB and A 0 (inner) -> 0x80 (outer) | radial noise streaks fading to the centre |
| 9803AD04 (dive 13) | 1.0 | flat ring r 0.37..0.50 | RGB 0..0x80, A 0x80 | thin bright rim at the outer edge |
| 9803AD0E (dive 14) | 1.0 | low dome r 0.22..0.50, y 0..0.09 | RGB 0..0x80, A 0x80 | nearly invisible at t 0; at t 8 a ring of blades (texture alpha only in u 0.125..0.5, swept by the u scroll -6 texels/frame) |
| 9803AD0D (dive 15, frames 0-20) | 1.0 | same dome | RGB 0..0x80, A 0x80 | raw v is 1.0 on every vertex: v comes only from the scroll (40 texels, v -13.8, a +3.5), so the whole dome shows one texture row band that changes over time; dark at t 0 and t 8 |
| 9803AD0C (dive 15, key at frame 21) | 2.0 | same dome, twice the size | as above | bright wide ring at t 8 |
| 9803990C (slam 2) | 0.998 | flat disc r 0.20..0.50 | RGB and A 0 -> 0x80 | streaks around a dark centre |
| 98039903 (slam 7, 8) | 1.0 | cup r 0.13..0.50, y 0.02..0.50 | RGB/A 0..0x70 | upward streaks |
| 9803990B (slam 9) | 0.99 | cone/ring rising r 0.25..0.50, y 0..0.25 | RGB/A 0..0x80 | swirling band |
| 9803990D (slam 10) | 0.986 | band r 0.40..0.50, y +-0.08, 3/4 circle | RGB 0..0x80, A 0x80 | open swirl |
| F000017C (slam 11) | 1.988 | cylinder r 0.24, y 0..0.50 (x s: r 0.49, height 1) | RGB 0x80, A 0..0x80 | wall fading along its height |

Checks this gives: the vertex formats and strides decode into clean closed shapes (no garbage
vertices); the radii match README section 12 once the 1/32767 scale and mesh scale are applied
(e.g. F000017C: README "radius 0.49, height 1"); the ramps 0 -> 0x80 make the inner edges fade out
with colour 0x80 = 1.0; the looks depend strongly on the UV scroll, so a port must animate it.
Not checked: the HD DDS art (`--hd` needs a Pillow that reads the DDS format) and the real K
(Colac x element colour) over time.
