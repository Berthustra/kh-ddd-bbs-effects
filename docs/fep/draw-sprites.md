# FEP sprites: FecAnm_plus (152) and FecMegaanm_plus (153)

Topic "sprites" of the FEP work. Everything here was read from the DDD Steam exe (KH 2.8, image
base 0x140000000, **every address is an RVA**) and from the files; neither game was run.
Labels as in [README.md](README.md): unmarked = read in the exe at the RVA given (and, where a
count is given, checked on the data); **INFERRED** = follows from read facts; **GUESS** = a guess.

Contents: 1 summary - 2 the ANM resource - 3 the ANM draw core (shared) - 4 FecAnm_plus -
5 FecMegaanm_plus - 6 the dive and the slam - 7 corrections to the other specs - 8 open points -
9 tools.

Conventions used below. Matrices are row-vector (`p' = p * M`): `0x63ad40(M, out, v)` computes
`out = v.x*M.row0 + v.y*M.row1 + v.z*M.row2 + v.w*M.row3`; `0x63ac90(A, out, B)` = `B * A`;
the 3x3 helpers `0x582810(out, a, b)` and `0x5b2de0(out, a, b)` (identical code) give
`out.3x3 = b.3x3 * a.3x3` with the translation row and the `w` column taken from `b`;
`0x58c5b0(out, X, Y)` = `Y * X`; `0x5b2f60(out, M, s)` scales row i of M by `s[i]` (= `diag(s) * M`).
`RotZ(out, in, deg)` (`0x565b50`) = `in * Rz`, `Rz.row0 = (cos, sin, 0)`, `Rz.row1 = (-sin, cos, 0)`
(`cosf` = import thunk `0x771c64`, `sinf` = `0x771c76`; degrees x 0.711111 x 0.0245437 = radians).
`rand01()` = `(rand() >> 1) * 2^-31` (`0x613020`, constant `[0x7ecf78]` = 4.65661e-10), in [0, 1).
`rnd(r)` = `rand01()*2r - r` (`0x58c5f0`), uniform in [-r, r). `rndp(r)` = `rand01()*r` (`0x58c630`).
`irand(n)` = `int(rand01()*n)` (`0x5811a0`). `dt` = the **element** instance's dt (`+0x13c`).

## 1. Summary

| | FecAnm_plus (152) | FecMegaanm_plus (153) |
|---|---|---|
| what | one ANM flip-book sprite per part, placed by the part's Lmatrix | a ring buffer of N sprites ("particles") living inside one part, each with its own ANM clock |
| params | 0x1f bytes + `_plus` byte `+0x1f` = **blend index** | 0x115 bytes + `_plus` byte `+0x115` = **blend index** |
| work | `+0` f32 time, `+4` u16 frame (0xFFFF = ended), `+6` u8 flip bits | `+0` -> buffer `0x30 + 0x80*N`, `+8` f32 emission accumulator, `+0xc` f32 last amount, `+0x10` s16 vertex counter |
| slots | 0 `0x586dc0` init, 1 `0x586c70` (`ret`), 2 `0x586c80` clock, 3 `0x586f10` draw | 0 `0x58df30` alloc, 1 `0x58cc40` key, 2 `0x58cdd0` emit + age, 3 `0x58e010` draw, 4 `0x58dfe0` free, 5 `0x58e7f0` (returns 1) |
| orientation | Lmatrix byte 1 & 7: 0 = the part's own axes, 1 = screen-aligned billboard, 2 = billboard turning about the frame's Y axis | always screen-aligned billboard (`[0xaa3e38]`) plus a per-sprite screen rotation |
| quad size | vertex x, y / 32767 x ANM size x 0.1, then the Lmatrix scale (percent) | vertex x, y / 32767 x ANM size x 0.001 x the sprite's scale (percent) x element scale |
| colour | input 0 (Col/Colac output, 0..255) x element colour, clamped 0..255 | per-sprite colour + velocity x age, fade in/out, x element colour, clamped |
| draw core | `0x55a360 -> 0x559eb0` (section 3) | same |

Both draw exactly one ANM frame per sprite through the same core. The core multiplies each quad's
own RGBA8 colour by the command's colour with `(c * C) >> 7` (so **128 = 1.0**, read at
`0x55a184..0x55a1f8`), binds the quad's texture and draws its 4 vertices as a strip.

## 2. The ANM resource (FEP section 4, resident.frr table 1)

Layout read from CAnm2D::Load `0x55a5f0` (relocation), the texture resolver `0x55a520`, the clocks
(`0x586c80`, `0x58dd9a..0x58de68`) and the draw core `0x559eb0`; field counts checked over all
1532 ANMs (every local ANM of the 667 FEPs + the 134 resident ones) with
`ddd_fep_sprites_anm.py check`. All offsets in the file count from the ANM start.

| off | type | meaning | read by |
|---|---|---|---|
| 0x00 | char[4] | `ANM\0` | |
| 0x04 | u32 | id (README 9.1) | manager |
| 0x08 | u16, u16 | 1, 2 (1532/1532) | none seen |
| 0x0c | u32 | 0 in the file; Load sets 1 and skips when already 1 (`0x55a5f9`, `0x55a609`) | Load |
| 0x10 | u32 | flags: **bit 7 = loop** (`test byte [anm+0x10], 0x80` at `0x586d34` and `0x58ddba`) | clocks |
| 0x14 | u32 | length in ticks = sum of the key durations (1532/1532) | clocks |
| 0x18 / 0x28 | u32 / u32 | texture count / offset: 8-byte `{u32 texture id, i32 slot}`; the slot is written by `0x55a5b0` from `find(owner, id)` | draw core `0x55a1fb..0x55a202` |
| 0x1c / 0x2c | u32 / u32 | key count / offset: 12-byte `{u32 flags, u32 index, u32 duration}` | clocks read **only the durations** (`+8`) |
| 0x20 / 0x30 | u32 / u32 | frame count / offset: 12-byte `{u32 flags, u32 n, u32 -> u32 quad index[n]}`; `+8` relocated by Load (`0x55a633`) | draw core reads `+4`, `+8` |
| 0x24 / 0x34 | u32 / u32 | quad count / offset: 0x1c-byte quads (below); `+0x0c` relocated (`0x55a661`) | draw core |
| 0x38 | f32 | **size**: the draws scale the quad by size x 0.1 (Anm_plus `0x586fdc..0x586fea`) or size x 0.1 x 0.01 (Megaanm_plus `0x58e11f..0x58e136`) | draws |

- **Key k shows frame k.** The clocks find the key index from the summed durations and pass that
  index straight to the draw core as the frame index (`0x586da8`, `0x58de59`; `0x559f0a..0x559f15`
  = frames + index x 12). Key `+4` equals its own index in 1532/1532 ANMs and key count = frame
  count in 1532/1532, so this is consistent; no code was seen reading key `+0` or `+4`.
- **Key flag 0x80**: set on the last key exactly when header bit 7 is set (1115 looping ANMs; the
  other 417 have no 0x80 key; no other key flag bits). The clocks test the header bit, not the key
  flag.
- **Frame flags** (`+0`): 0 (1707), 8 (9203), 0x80 (734), 0x88 (384) game-wide. The draw core
  never reads them (it reads frame `+4` and `+8` only, `0x55a101`, `0x55a134`). Meaning unknown,
  not needed to draw.
- **Quad** (0x1c bytes): `+0x00` u32 index into the ANM's texture list (always < count, 19863
  quads); `+0x04` u32 **RGBA8 colour** (byte 0 = R ... byte 3 = A; multiplied in, section 3);
  `+0x08` u32 vertex count (4 in 19863/19863); `+0x0c` -> vertices; `+0x10` u32 blend code (read
  only when the command's blend index is 4: `0x55a14d..0x55a155`); `+0x14`, `+0x18` = (0, -1)
  with codes 0xA0600 / 0xA0602 and (0xFFFFFFFF, 0) with 0x70600; no reader seen.
- **Vertex** (10 bytes): `s16 u, s16 v, s16 x, s16 y, s16 z`. The converter `0x66baf0` (installed
  for the effect pipeline, reached from the draw `0x5e33f0 -> 0x66ff70 -> 0x671980 -> [pipe+0x258]`)
  computes `x, y, z * 1/32767` (`[0x81b49c]` = 3.05185e-05) and `u, v * 1/16384` (`[0x81b4a0]` =
  6.10352e-05), all **signed** (`movsx`), and uses `1 - v` **unless the pipeline type is 5**
  (`cmp [pipe+0x18], 5` at `0x66bc16`, `0x66bccc`, ...). The sprite draws bind CVShaderEffect
  (`[0xaaa950]`, vtable `0x7f42a0`, RTTI `.?AVCVShaderEffect@Graphics@System@Som@@`), whose method
  `0x600ef0` sets its type word `+0x10` = 5 (`mov [rcx+0x10], 0x10005`), and `0x601480 -> 0x670130`
  copies that word into the pipeline type (`0x670151`). So **v is used as stored**: texture
  coordinate `(u/16384, v/16384)` with v = 0 at the first decoded row (the decoder writes no flip,
  README 9.3). z = 0 in every vertex (19863 quads).
- **Corner order**: TL, TR, BL, BR (in 16880 of the 19863 quads the four corners form an
  axis-aligned rectangle in that order, with x growing right, y growing up, u right, v down in
  16190 of them; the rest are rotated or mirrored cells). Drawn as primitive 6 (`0x55a293`) with 4
  vertices: **INFERRED a triangle strip** (`0x66baf0` treats primitive 6 specially, `0x66bb23`;
  a strip is the only reading that fills the rectangles).

## 3. The ANM draw core (`0x55a360 -> 0x559eb0`), shared by both classes

`0x55a360(anm_mgr, handle, frame, M, &colour, blend_index, f32 depth_offset, u8 keep_depth)`
looks up the CAnm2D object `[mgr + 0x30 + handle*0x20]` and jumps to `0x559eb0`. Per call:

1. Frame record = `frames + frame*12` (`0x559f0a..0x559f19`); no range check (both clocks keep the
   index valid).
2. The 4x4 world matrix M is uploaded as a 3x4 (its columns as rows, `0x559f55..0x55a015`,
   `0x601b00`) for the effect shader `[0xaaa950]` (bound by `0x601480`, which also passes
   (128, 128, 128, 128) as the program's colour scale because its `+0x12` = 1, `0x6014a6`;
   **INFERRED**: colour bytes are read with 128 = 1.0 in the shader as well).
3. Depth offset: if the f32 argument is not 0, `0x5e3090(state, offset, [[0xaa3e40]+0x28])` before
   and `0x5e3090(state, 0, 0)` after (`0x55a05e..0x55a077`, `0x55a2e1..0x55a2f1`); `[0xaa3e40]` is
   the projection matrix, `+0x28` its m22. What `0x66fe00` does with it: not read (see 8).
4. `keep_depth` byte: both classes pass 1. With 0 the core would set state `+0x80 |= 0x20` and
   clear `+0x40` for the draw (**GUESS** depth write off).
5. Blend: if `blend_index != 4`, mode = `[0x7ed8e0 + 4*index]` (`0x5c02d0`): index 0 -> 1,
   1 -> 2, 2 -> 3, 3 -> 1; it is set once (`0x5e2300`). If `blend_index == 4`, each quad's code
   `+0x10` picks the mode (`0x5c02a0`: 0x70600 -> 1, 0xA0600 -> 2, 0xA0602 -> 3, else 0), set
   again when it changes. `0x5e2300(state, mode)`: 0 = blending off; else enable with
   `+3..+4` = 05 02, `+6` = 1, and mode 2 -> `+1..+2` = 01 01, `+5` = 2; mode 3 -> 03 03, 2;
   mode 1 and others -> 01 01, 6. The D3D meaning of those bytes is the render-state topic's.
   **In all Flowmotion data the result is mode 2**: blend index 1 (58 of 62 Anm_plus blocks, 49
   of 49 Megaanm_plus blocks) -> table -> 2, the same mode the quads' own code 0xA0600 gives;
   4 Anm_plus blocks use index 3 -> mode 1.
6. Colour: `C = trunc(colour)` per channel (`cvttps2dq`, `0x55a110`; the callers clamp it to
   0..255 first). Per quad: `R' = min(255, (quad.R * C.r) >> 7)`, same for G, B, A (unsigned,
   `0x55a184..0x55a1f8`); the packed `A'B'G'R'` is set as the constant colour through
   `0x5e29a0` when it changes. The vertices carry colour 0xFFFFFFFF (`0x66bbea`).
   **INFERRED** (texture combine not read): output = texture x constant colour.
7. Texture: slot = ANM texture list `[quad+0].slot` (`0x55a1fb..0x55a202`), bound by
   `0x5c0710(texmgr, ..., slot, 5, 1, 0, 0)` when it changes.
8. Vertices: `0x6007c0` sets the quad's vertex pointer, `0x5e33f0(.., 6, 0, count)` draws.

## 4. FecAnm_plus (152)

### 4.1 Parameter block (0x1f bytes + `+0x1f`)

Fields read by the class's code (`0x586c70..0x587321`) and `0x5b39a0`. "FM" = values in the 62
Anm_plus blocks of the 19 Flowmotion elements (init + key blocks).

| off | type | meaning | read at | FM values |
|---|---|---|---|---|
| 0x00 | f32 x3 | depth-offset vector (y, z negated on read, `0x5b3a39..0x5b3a64`); only with `+0x1d` | `0x5b3a22` | (0,0,0), (0,0,-0.8), (0,0,-0.7), (0,0,-0.5), (0,0,+0.5) |
| 0x0c | s32 | section-4 (ANM) index; < 0 = draw nothing, clock idle | `0x586ca9`, `0x586f3e` | |
| 0x10 | f32 | animation speed (ANM ticks per frame) | `0x586d16` | 0 (4), 1 (48), 2 (10) |
| 0x14 | f32 | depth-offset scale s: the offset matrix uses scale (s, s, s) x size x 0.1 | `0x5871c8` | 1.0 |
| 0x18 | u16 | random start: initial time += rand01 x value | `0x586e09` | 0 |
| 0x1a | u8 | flip mode: 1 = bit 0 random (50 %), 2 = bit 1 random, 3 = `int(rand01*255) >> 6` (0..3) | `0x586e3a` | 0 |
| 0x1b, 0x1c | u8 | no reader in the class code; 0 in 22226/22226 blocks game-wide | | 0 |
| 0x1d | u8 | depth offset on | `0x5871be` | 0 (14), 1 (48) |
| 0x1e | u8 | depth-offset space: 0 part-local, 2 world, 3 view (`0x5b3af9..0x5b3c60`) | `0x5871d4` | 3 |
| 0x1f | u8 | `_plus` field = **blend index** (section 3 step 5) | `0x5872c4` | 1 (58), 3 (4) |

Inputs (the slot descriptor's input list): input 0 = the Col/Colac command (its work output vec4
is the colour, `0x5870e5..0x587116`), input 1 = the Lmatrix (its parameter bytes 0, 1, 2 are
read, `0x58700e..0x5870cb`).

### 4.2 Clock (slots 0, 1, 2)

```
slot 0 (0x586dc0), at spawn:                       work: +0 f32 t, +4 u16 frame, +6 u8 flip
    frame = 0
    t = -dt                                         (element dt, 0x586df2..0x586e01)
    if p.rand_start: t += rand01() * p.rand_start   (0x586e1c..0x586e36)
    flip = by p.flip_mode (4.1)                     (0x586e3a..0x586ee7)
    return 1
slot 1 (0x586c70): ret 0 -- a key block does not touch the clock
slot 2 (0x586c80), every frame (pass-1 update):
    if frame == 0xFFFF: return                      (ended)
    frame = 0
    if p.anm < 0 or no ANM record / handle -1: return
    t += dt
    s = p.speed * t;  L = anm.length
    if s >= L:
        if not (anm.flags & 0x80): frame = 0xFFFF; return       (0x586d34..0x586d4a)
        s = float(int(s) mod int(L));  t = s / p.speed          (0x586d4b..0x586d64)
    k = 0; end = key[0].duration
    while s >= end: k += 1; end += key[k].duration              (0x586d6d..0x586da6)
    frame = k
```
Notes. The wrap drops the fraction of `s` (integer `idiv`) and stores the **unscaled** time back
(commands.md 6.7 says the scaled value is stored; that is FecMegaanm_plus, 5.4). With speed 0 the
sprite stays on frame 0 (slam parts 0, 1). The first update after spawn gives t = 0. The draw is
skipped while frame = 0xFFFF, so a non-looping ANM simply disappears after its last key even if
the part lives on.

### 4.3 Draw (slot 3, `0x586f10`)

```
if work.frame == 0xFFFF or p.anm < 0 or ANM handle == -1: return
f      = anm.size * 0.1                                         (0x586fdc..0x586fea)
F      = flip matrix(work.flip)  (0x5bce80: identity; bit 0 -> RotY 180; bit 1 -> RotX 180;
                                  3 = RotY 180 then RotX 180)
M      = 0x582810(F, local)       -> M.3x3 = local.3x3 * F.3x3, M.t = local.t   (0x586ff7..0x587009)
         (local = part instance +0x78, the Lmatrix output S*R*T)
lm     = parameter bytes of input 1 (the Lmatrix): b0 = attach mode, b1, b2
if (b1 & 7) == 1:
    M.3x3 = M.3x3 * cam[elem.byte1].m160.3x3                    (0x58704a..0x587070)
    (cam = [0xaa3e10 + 8*i]; +0x160 = identity as set every frame by 0x557570 -> 0x5bb780,
     whose 5th argument is null at 0x5576ae)
M      = world(ctx, part, M, b0, b1, b2, f)      (0x5b2190 = jmp 0x5b1510; 4.4)
C      = input0.work[0..3] * elem.colour (+0x40), each clamped to [0, 255]   (0x587116..0x5871b6)
depth  = 0
if p.depth_on (+0x1d):                                          (0x5871be..0x5872b1)
    Q     = 0x5b39a0(ctx, part, local, b1, f, p.depth_space, p, (s,s,s,1), out)   (4.5)
    zlen  = |frame.row2.xyz|   (frame = owner part +0x88, else element +0x60)
    depth = 0x5baa40(M, Q, zlen)
draw_core(anm, work.frame, M, C, p.blend (+0x1f), depth, keep_depth = 1)    (0x5872e7)
```
`elem.colour` (`+0x40`) is the element's colour multiplier: rgb copied from `+0xd0` (default 1.0,
`0x5ac644`, `0x5ae632..0x5ae661`), alpha `+0x4c` = stop fade `+0x148` x `+0xdc` x the view /
distance factor (`0x5b49e2..0x5b49fd`, `0x5b4ff0..0x5b500e`, `0x5b32f5`). So the 30/15-frame stop
fade of README 5 reaches the sprites through this alpha.

### 4.4 The world matrix for sprites (`0x5b1510`, by Lmatrix byte 1 & 7)

Arguments as passed by the Anm_plus draw: `M` (above), `f` = ANM size x 0.1; `frame` = the owner
part's `+0x88`, else the live effect matrix element `+0x60` (`0x5b152e..0x5b154e`). Camera globals,
filled every frame by `0x557570 -> 0x5bb780` from camera object `[0xaa3e10]`: `[0xaa3e30]` = view
V (`cam+0x60`), `[0xaa3e38]` = inverse view rotation (`cam+0xa0`, the transpose of V's 3x3 with
the eye position, `0x5bb8bf..0x5bba0b`), `[0xaa3e40]` = projection P (`cam+0x20`), `[0xaa3e48]` =
V*P (`cam+0xe0`, `0x5bb805..0x5bb832`).

| b1 & 7 | read at | result (`p_world = v * M'`) | in Flowmotion Anm_plus |
|---|---|---|---|
| 0 | `0x5b1579..0x5b15be` | `M' = [M.3x3 * f ; M.t] * frame` : the quad lies in the part's own XY plane, rotated by its Rot and carried by the frame (no billboard) | 26 blocks (dive SHO0 parts 0-3, slam SHO0 parts 0, 1) |
| 1 | `0x5b15db..0x5b183f` | `M'.3x3 = diag(sx,sy,sz) * M.3x3 * f * Vinv.3x3`, `M'.t = M.t * frame`, with `s_i = max(1e-5, |frame.row_i|)` (`0x5b1764..0x5b17de`): **screen-aligned billboard**; the part's rotation spins/tilts the quad in camera space | 34 blocks (e.g. slam part 6, all CAL0/DRL0/ASK/COM sprites) |
| 2 | `0x5b1dc9..0x5b2133` | `A = frame.3x3 * V.3x3`; `y = normalize(A.row1)`; `x = normalize(y.y, -y.x, 0)`; `z = normalize(x cross y)`; `A = [x; y; z] * Vinv.3x3`; `M'.3x3 = diag(elem.scale) * M.3x3 * A * f`; `M'.t = M.t * frame`: **billboard that keeps the frame's Y axis** and turns about it to face the screen | 2 blocks (slam part 4) |
| 3, 4 | `0x5b1844..0x5b1dc4` | camera-based variants (atan2 of the view position, `0x771c58`; `0x5afb10`, `0x5afc50`, `0x63d030`) | none; not decoded |
| any, bit 3 (8) set | `0x5b15fb`, `0x5b1864`, `0x5b1a72`, `0x5b1dfa` | first builds a rotation from the frame position seen in view space | none (game-wide value 11 in 12 blocks); not decoded |

Mode 0 also applies the frame's scale (it multiplies by the full frame matrix); mode 1 applies the
frame's axis lengths, mode 2 the element scale `+0x30` instead.

### 4.5 Depth offset (`0x5b39a0` + `0x5baa40`)

The "bias" moves only the sprite's **depth**, not its picture:

```
o   = p[0x00..0x08] with y, z negated            (FM: (0, -0, +0.8) etc.)
S   = diag(f * s, f * s, f * s)                   (s = p+0x14)
camera modes 1 / 2: separate branches 0x5b405a / 0x5b3c65 (not read; same translation INFERRED)
other modes: Q = [local.3x3 * S ; local.t]
    depth_space 0: Q.t += o;  Q = Q * frame * (V*P)          (0x5b3bf8..0x5b3c60)
    depth_space 2: Q = Q * frame;  Q.t += o;  Q = Q * (V*P)   (0x5b3b8b..)
    depth_space 3: Q = Q * frame * V;  Q.t += o;  Q = Q * P   (0x5b3b15..0x5b3b86, 0x5b420c)
0x5baa40(M, Q, zlen):                                         (0x5baa40..0x5baab7)
    w0 = (M.t * (V*P)).w                          clip w of the sprite's origin
    w1 = w0 + (Q.row3.w - w0) * zlen              clip w of the pushed point, scaled by frame z scale
    n  = [[0xaa3e00] + 0x1ac] * 1.1               (GUESS: near plane distance)
    if w0 > w1 and w0 >= n and n > w1: w1 = min(w0, n)
    return w1 - w0                                (negative = toward the camera)
```
In the Flowmotion data the offset is a view-space z of +0.5..+0.8 FEP units (mode 3), i.e. the
sprite is depth-tested as if 0.5..0.8 units nearer the camera (**INFERRED**: so ground flashes do
not sink into the floor or the character), never closer than 1.1 x the near value.
**The sign is kept**: the aura's sprites (ORA0, ORA1, FOR0 parts 0-1, W_010_SO_ORA0_0) store
+0.3 and FIR0 parts 0/1/12 +0.5, i.e. view z -0.3 / -0.5: tested *farther* from the camera,
behind the limbs they sit on (aura.md 7.3).

## 5. FecMegaanm_plus (153)

A ring buffer of up to N sprites ("sparks") owned by one part. Each sprite is spawned once with a
start position, velocity, acceleration, scale, spin and colour, all **closed-form in its age**
(nothing is integrated per frame): the update slot only emits, ages and retires them; the draw
slot evaluates each live one and draws it with the ANM draw core (section 3). Read in full:
slot 0 `0x58df30`, 1 `0x58cc40`, 2 `0x58cdd0`, 3 `0x58e010`, 4 `0x58dfe0`, 5 `0x58e7f0`, and the
helpers `0x58cb70`, `0x5b21a0`, `0x58e800`, `0x582b20`, `0x58c660`, `0x63b900`, `0x58c6c0`,
`0x58c510`, `0x63b0c0`, `0x63b890`, `0x63c240`, `0x5b2d70`. Data: `ddd_fep_sprites_mega_params.py`
(49 blocks in the 19 Flowmotion elements, 16428 game-wide). "FM" = the 49 Flowmotion blocks.

Extra helpers. `RotXYZ(a)` (`0x58c660`) = identity, then `RotZ(a.z)`, `RotY(a.y)`, `RotX(a.x)`
(each `in * R`), i.e. `A = Rz * Ry * Rx` (row vectors; RotX rows (1,0,0),(0,c,s),(0,-s,c), RotY
(c,0,-s),(0,1,0),(s,0,c), RotZ (c,s,0),(-s,c,0),(0,0,1), degrees, commands.md 6.2).
`cross(a, b)` (`0x63b900`) = a x b. `norm(v)` (`0x58c6c0`) = v / |v|, zero stays zero, w = 1.
`LookAt(t, d)` (`0x582b20`): `up = (0,0,1)` if `d.x == 0 and d.z == 0` else `(0,1,0)`;
`X = norm(cross(up, d))`, `Z = norm(d)`, `Y = cross(Z, X)`; rows `X, Y, Z, (t.xyz, 1)`, w column 0.
`xform(v, M)` (`0x58c550`) = `(v.xyz, 1) * M`. `frame()` (`0x58c510`) = the frame owner's `+0x88`,
else the element's effect matrix `+0x60`. `L` = the local matrix the part instance points to at
`+0x78`. `mt` = part instance `+0x90` (matrix type, README 7.3).

### 5.1 Parameter block (0x115 bytes + `_plus` byte `+0x115`)

Every offset the class reads is listed. Unread: `+0x10f` (3 in 16428/16428), `+0x113` (0),
`+0x114` (0/1). "FEP space" vectors get y and z negated by the code (engine space, README 8.2).

| off | type | meaning | read at | FM values |
|---|---|---|---|---|
| 0x00 | f32 x3 | shapes 0, 2: half-size of the spawn box. Shapes 1, 3, 6, 7: `+0x00` = radius R | `0x58d13c`, `0x58d29d`, `0x58d2e2` | box (0.2,1,0.2), (0.1,1.2,0.1); R 1.3, 4 |
| 0x0c | s32 | section-4 ANM index (< 0: no draw and no ageing) | `0x58dd05`, `0x58e055` | |
| 0x10 | f32 x3 | cone axis, FEP space (position cone and velocity cone) | `0x58d213`, `0x58d76b` | (0,1,0), (0,-1,0) |
| 0x1c | f32 | ANM speed (ANM ticks per frame of age) | `0x58dda8` | 1, 2 |
| 0x20 | f32 x3 | second velocity direction, FEP space | `0x58d8f9` | (0,-1,0), (0,0,-1) |
| 0x2c | f32 | emission rate (sprites per frame) | `0x58cc5d`, `0x58cf21` | 0..3 |
| 0x30 | f32 x3 | random +- added to `+0x20` (with `+0x106`) | `0x58d932` | 0 |
| 0x3c | f32 | rate "acceleration": < 0 ends emission at t = rate / -acc; also in the amount (5.3) | `0x58cc70`, `0x58ced9`, `0x58cf91` | 0 (49); < 0 in 4 blocks game-wide |
| 0x40 | f32 x4 | colour RGBA at spawn (128 = 1.0) (with `+0x10c`) | `0x58dbac` | |
| 0x50 | f32 x4 | colour change per frame of age (with `+0x10c`) | `0x58dbb3` | |
| 0x60 | f32 x4 | random +- added to `+0x40` (with `+0x10d`) | `0x58dbed` | 10..20 rgb, 0 a |
| 0x70 | f32 x4 | random +- added to `+0x50` (with `+0x10d`) | `0x58dbff` | 0 |
| 0x80, 0x84 | f32 x2 | scale x, y in percent (with `+0x107`; else 100) | `0x58dab5` | 8..28 |
| 0x88, 0x8c | f32 x2 | scale velocity per frame | `0x58dabd` | 0 |
| 0x90, 0x94 | f32 x2 | scale acceleration | `0x58dac6` | 0 |
| 0x98, 0xa0, 0xa8 | f32 x2 each | random for scale / scale vel / scale acc (with `+0x108`, mode `+0x109`; `0x58e800`) | `0x58daf3` | 3..10 |
| 0xb0 | f32 | cone half-angle, degrees (both cones) | `0x58d1b8`, `0x58d719` | 180 |
| 0xb4, 0xb8 | f32 | speed, acceleration along the first direction (with `+0x103`) | `0x58d677` | (0.15,-0.01), (0,0) |
| 0xbc, 0xc0 | f32 | random +- for them (with `+0x104`) | `0x58d6d3` | |
| 0xc4, 0xc8 | f32 | speed, acceleration along `+0x20` (with `+0x105`) | `0x58d90e` | (0,0.008), (-0.05,0.001), (0.05,0) |
| 0xcc, 0xd0 | f32 | random +- for them (with `+0x106`) | `0x58d980` | (0,0.002), (0.015,0) |
| 0xd4, 0xd8, 0xdc | f32 x3 | screen angle (deg), spin (deg/frame), spin acceleration (with `+0x10a`) | `0x58db3a` | (0..840, 70, 0) |
| 0xe0, 0xe4, 0xe8 | f32 x3 | random +- for them (with `+0x10b`) | `0x58db5e` | (180, 0, 0) |
| 0xec | f32 | depth offset, view z, subtracted (negative = toward the camera); with `+0x10e`, off when 0 | `0x58e148`, `0x58e6bc` | -0.3, -0.5 |
| 0xf0 | u16 | N = capacity (only the **init** block's, slot 0) | `0x58df56` | 5..15 |
| 0xf2 | u16 | life in frames of age | `0x58cc98` | 13..20 |
| 0xf4 | s16 | shapes 8, 9: section-3 MDL index | `0x58d370` | -1 |
| 0xf6, 0xf8 | s16 | shapes 8, 9: VTL record index, VTL list index (as Generateanchorvertex, generators.md 5.3) | `0x58d444` | -1 |
| 0xfa | u8 | flip mode: 1 = bit 0 random, 2 = bit 1 random, 3 = irand(255) >> 6 | `0x58d0b7` | 0 (33), 3 (16) |
| 0xfb | u8 | space (5.4): 7 = world at spawn (with mt 0/3/4); 2, 6, 7 = positions taken as world by the draw; 0xff = part-local | `0x58d181`, `0x58e1c2`, `0x58dfb4` | 255 (32), 7 (17) |
| 0xfc | u8 | interval: each emitting frame subtracts `interval` frames of amount from the accumulator | `0x58cf8a` | 0, 1, 2 |
| 0xfd | u8 | chance in % per sprite (>= 100: no draw) | `0x58d025` | 100 (48), 70 (1) |
| 0xfe | u8 | shape: 0 box + cone velocity, 1 cone + cone vel, 2 box + radial vel, 3 cone + radial, 6 arc + cone vel, 7 arc + radial, 8 MDL vertex + cone vel, 9 MDL vertex + radial; 4, 5, > 9 write no position | jump tables `0x58ded8`, `0x58df00` | 0 (17), 1 (3), 2 (7), 3 (8), 6 (3), 7 (11) |
| 0xff | u8 | shapes 8, 9: 2 = vertices in order (cursor work `+0x10`), else random | `0x58d4b4` | 0 |
| 0x100 | u8 | radius mode: 0 = uniform in [0, R); else by `+0x101` | `0x58d294` | 0 (25), 1 (24) |
| 0x101 | u8 | 0: R - rndp(`+0x102`/100) x R; 1: rndp(`+0x102`/100) x R; other: radius = `+0xb0` (stale register) | `0x58d2b7` | 0 (42), 1 (7) |
| 0x102 | u8 | percent for `+0x101` | `0x58d2c7` | 0, 12, 15, 25 |
| 0x103 .. 0x10e | u8 | enables (see the rows above): `103` speed/acc, `104` their random, `105` speed2/acc2, `106` dir2 + speed2 random, `107` scale, `108` scale random, `109` its mode, `10a` rotation, `10b` rotation random, `10c` colour, `10d` colour random, `10e` depth offset | | |
| 0x110 | u8 | fade on | `0x58ccb8` | 0 (24), 1 (25) |
| 0x111 | u8 | fade-in frames | `0x58cccd` | 0, 2 |
| 0x112 | u8 | fade-out frames | `0x58ccc5` | 0, 3, 4 |
| 0x115 | u8 | `_plus` field = blend index (section 3 step 5) | `0x58e720` | 1 (49) |

Everything except N and the alloc-time world flag is read from the **current** block (keys
change it), including the ANM, its speed, life, fades and the draw fields, so a key retunes the
live sprites too.

### 5.2 Buffer (slot 0, effect allocator `0x5bd400`, **not** zeroed)

Header (0x30 bytes):

| off | type | meaning | written |
|---|---|---|---|
| 0x00 | | unused | |
| 0x04 | f32 | emission end: 0 = none (rate <= 0); -1 = forever (rate_acc >= 0); else rate / -rate_acc | key `0x58cc90` |
| 0x08 | f32 | life (`+0xf2`) | key `0x58ccab` |
| 0x0c | f32 | fade-out start age | key |
| 0x10 | f32 | fade-in rate (1/frames) | key |
| 0x14 | f32 | fade-out rate | key |
| 0x18 | f32 | time since the last key, += dt every update | key (0), `0x58cebb` |
| 0x1c | s16 | N | slot 0 `0x58df84` |
| 0x1e | s16 | live count | slot 0 (its dword write also zeroes `+0x20`), update |
| 0x20 | s16 | head = index of the oldest sprite | |
| 0x22 | u8 | velocity-to-world flag: init `+0xfb` == 7 and mt in {0, 3, 4} (`0x58df96..0x58dfc3`) | slot 0 |
| 0x23..0x2f | | unused | |

Sprite i (i = 0 .. count-1, oldest first) is at `0x30 + 0x80 * ((head + i) mod N)`.

Record (0x80 bytes):

| off | type | meaning |
|---|---|---|
| 0x00 | f32 x4 | colour at spawn (RGBA) |
| 0x10 | f32 x4 | colour velocity |
| 0x20 | f32 x3 | position at spawn (part-local or world, 5.4) |
| 0x2c | f32 x3 | velocity per frame of age |
| 0x38 | f32 x3 | acceleration |
| 0x44 | f32 x3 | scale x, y (percent); z = 100 |
| 0x50 | f32 x3 | scale velocity x, y; z = 0 |
| 0x5c | f32 x3 | scale acceleration x, y; z = 0 |
| 0x68 | f32 | age (frames) |
| 0x6c | f32 x3 | screen angle (deg), spin, spin acceleration |
| 0x78 | f32 | ANM time |
| 0x7c | u16 | ANM frame (key index) used by the draw |
| 0x7e | u8 | flip bits |
| 0x7f | | unused |

Work: `+0x00` buffer pointer, `+0x08` f32 emission accumulator, `+0x0c` f32 last emission amount
(write-only; the key stores the rate there), `+0x10` s16 in-order vertex cursor (shapes 8, 9 with
`+0xff` == 2).

### 5.3 The slots

```
slot 0 (0x58df30) init:
    work.buf = alloc(0x30 + 0x80 * p.N)    (fails -> return 0: the part is not spawned)
    hdr.N = p.N; hdr.count = 0; hdr.head = 0; work.acc = 0; work.last = 0; work.cursor = 0
    hdr.world_vel = (p.space == 7 and mt in {0, 3, 4})
    return 1
slot 4 (0x58dfe0): free work.buf, null it.
slot 5 (0x58e7f0): return 1. A loop rewind changes nothing: buffer, accumulator, key clock persist.

slot 1 (0x58cc40) key, new block p:   (nothing if no buffer, or if part state & 2 = has looped)
    hdr.emit_end = 0 if p.rate <= 0 else (-1 if p.rate_acc >= 0 else -(p.rate / p.rate_acc))
    hdr.t_key = 0; hdr.life = float(p.life); work.last = p.rate
    if not p.fade_on: hdr.fade_in = 1.0; hdr.fade_out = 0; hdr.fade_start = 0
    else (integers fi = p.fade_in, fo = p.fade_out, L = p.life):
        if fi + fo < L: fade_start = L - fo; fade_out = fo ? 1/fo : 0; fade_in = fi ? 1/fi : 1.0
        elif fo < L:    fade_start = L - fo; fade_in = 1/(L - fo);   fade_out = fo ? 1/fo : 0
        else:           fade_start = 0;      fade_in = 0;            fade_out = 1/fo

slot 2 (0x58cdd0) update, dt = element dt (+0x13c):
    if no buffer or dt == 0: return
    hdr.t_key += dt
    --- emission (0x58ce92..0x58dcb0) ---
    if hdr.emit_end == 0 or (hdr.emit_end > 0 and hdr.t_key > hdr.emit_end): goto ageing
    amount = dt*dt*(p.rate_acc*0.5) + dt*p.rate;  work.last = amount
    if amount <= 0: goto ageing
    s = work.acc + amount
    if s < 1: work.acc = s; goto ageing
    n = int(s);  step = dt / s
    work.acc = (s - n) - (p.interval*amount + p.interval*p.interval*(p.rate_acc*0.5))
    for r = n down to 1:
        if p.chance < 100 and not (p.chance > int(rand01()*100)): continue
        if hdr.count >= hdr.N: break               (full: the rest of this frame's sprites are lost)
        rec = record (head + count) mod N;  count += 1
        rec.frame = 0;  rec.age = rec.anm_t = r*step - 1.0
        rec.flip = p.flip == 1 ? (irand(256) >= 128 ? 1 : 0)
                 : p.flip == 2 ? (irand(256) >= 128 ? 2 : 0)
                 : p.flip == 3 ? irand(255) >> 6 : 0
        spawn position, then velocity, scale, rotation, colour (below)
    --- ageing (0x58dcfe..0x58de93), also on frames without emission ---
    if p.anm < 0 or no section-4 table or handle == -1: return     (sprites never age or die)
    dead = 0
    for i in 0 .. count-1: rec = record (head + i) mod N
        if rec.age >= hdr.life: dead += 1; continue
        t = rec.anm_t;  x = t * p.anm_speed;  Ln = float(anm.length)
        if x >= Ln:
            if not (anm.flags & 0x80): dead += 1; continue
            x = x - float(int(x) - int(x) mod int(Ln));  t = x          (scaled value kept)
        k = 0; end = key[0].dur; while x >= end: k += 1; end += key[k].dur
        rec.frame = min(k, keycount - 1)
        rec.age += dt;  rec.anm_t = t + dt
    head = (head + dead) mod N;  count = max(0, count - dead)      (the `dead` oldest are dropped)
```

Spawn position (`0x58d119`, table `0x58ded8`, by `p.shape`):

```
0, 2 (box):  P = (rnd(p.box.x), rnd(p.box.y), rnd(p.box.z))
1, 3 (cone): a = (rnd(p.cone_deg), rnd(p.cone_deg), rnd(p.cone_deg))      draw order x, y, z
6, 7 (arc):  a = (0, rnd(p.cone_deg), 0)
  then:      ax = p.axis; if ax.x == 0 and ax.y == 0: ax.x = 1e-6;  ax.y = -ax.y; ax.z = -ax.z
             B = RotXYZ(a) * LookAt(0, ax);  R = p[0x00]
             radius = p[0x100] == 0 ? rnd(R*0.5) + R*0.5
                    : p[0x101] == 0 ? R - rndp(p[0x102]/100) * R
                    : p[0x101] == 1 ? rndp(p[0x102]/100) * R
                    : p.cone_deg                                   (stale register; not in data)
             P = radius * B.row2
8, 9 (MDL):  one vertex of a VTL list as Generateanchorvertex (generators.md 5.3):
             j = p[0xff] == 2 ? cursor (work +0x10, then +1, wraps at n) : irand(n)
             P = vertex * 2^-15 (minus the mesh centre +0x38 if MDL record +0x14 & 1)
                 * MDL record f32 +0 * mesh f32 +0x24
             any missing piece: rec.pos = 0, d = (0,0,0), skip `place` (0x58d694)
4, 5, > 9:   nothing: rec.pos keeps old memory (never in the data)
place (0x58cb70 -> 0x5b21a0):
    if p.space == 7 and mt in {0,3,4}: W = L * frame(); pos = xform(P, W)
                                        d = norm(pos - W.t) * (|W.row0|, |W.row1|, |W.row2|)
    elif mt == 2 or mt in 6..9:        W = L * frame(); pos = xform(P, W); d = norm(pos - W.t)
    else:                              pos = P; d = norm(P)        (FM: mt 0, 1, 3, space 255)
    rec.pos = pos.xyz
```

Spawn velocity, scale, rotation, colour (`0x58d656..0x58dc8b`), in this order:

```
rec.vel = rec.acc = 0
spd, acc = p[0x103] ? (p.spd, p.acc) : (0, 0)
if p[0x104]: spd += rnd(p.spd_rnd); acc += rnd(p.acc_rnd)
by p.shape (table 0x58df00):
  0, 1, 6, 8 (cone velocity):
      a = (rnd(p.cone_deg), -rnd(p.cone_deg), -rnd(p.cone_deg))          draw order x, y, z
      ax = p.axis with y, z negated; if ax.x == 0 and ax.y == 0: ax.x = 1e-6
      B = RotXYZ(a) * LookAt(0, ax)
      if hdr.world_vel: B.3x3 = B.3x3 * L.3x3;  B.3x3 = B.3x3 * frame().3x3   (0x582810 x2)
      u = 1e-5 * B.row0 + B.row2                                         (not normalised)
      rec.vel = u * spd;  rec.acc = u * acc
  2, 3, 7, 9 (radial): rec.vel = d * spd;  rec.acc = d * acc             (d from `place`)
  4, 5, > 9: nothing
e = p.dir2;  spd2, acc2 = p[0x105] ? (p.spd2, p.acc2) : (0, 0)
if p[0x106]: e.x += rnd(p.dir2_rnd.x); e.y += rnd(..y); e.z += rnd(..z)
             spd2 += rnd(p.spd2_rnd); acc2 += rnd(p.acc2_rnd)
e = norm(e);  v2 = e*spd2;  a2 = e*acc2;  negate y, z of v2 and a2
rec.vel += v2;  rec.acc += a2              (never rotated: the axes of the sprite's own space)
rec.scale = (100,100,100); rec.scale_vel = rec.scale_acc = 0
if p[0x107]: rec.scale.xy = p.scl; rec.scale_vel.xy = p.scl_vel; rec.scale_acc.xy = p.scl_acc
if p[0x108]:                                                            (0x58e800, 2 components)
    mode p[0x109] == 0: rb = rnd(p.scl_rnd.x); rv = rnd(p.scl_vel_rnd.x); ra = rnd(p.scl_acc_rnd.x)
                        for c in x, y: scale[c] += rb; scale_vel[c] += rv; scale_acc[c] += ra
    mode 1: for c in x, y:
                scale[c] += scale[c]*0.01*rnd(p.scl_rnd[c])
                scale_vel[c] += scale_vel[c]*0.01*rnd(p.scl_vel_rnd[c])
                scale_acc[c] += scale_acc[c]*0.01*rnd(p.scl_acc_rnd[c])
                scale[c] += rnd(p.scl_rnd[c]); scale_vel[c] += rnd(p.scl_vel_rnd[c])
                scale_acc[c] += rnd(p.scl_acc_rnd[c])
    other modes: nothing
rec.rot = p[0x10a] ? p.rot : (0,0,0);  if p[0x10b]: rec.rot[k] += rnd(p.rot_rnd[k]), k = 0, 1, 2
if p[0x10c]: rec.col = p.col; rec.col_vel = p.col_vel
else:        rec.col = (128,128,128,128); rec.col_vel = 0
if p[0x10d]: for c in r, g, b, a: rec.col[c] += rnd(p.col_rnd[c]); rec.col_vel[c] += rnd(p.col_vel_rnd[c])
```

### 5.4 The draw (slot 3, `0x58e010`)

```
if no buffer or p.anm < 0 or no section-4 table or handle == -1: return
k = anm.size * 0.1 * 0.01                                             (0x58e11f..0x58e136)
depth_on = p[0x10e] and p.depth != 0;  if depth_on: zlen = |frame().row2.xyz|   (0x58e13f..0x58e1b4)
pos_is_world = p.space in {2, 6, 7}                                   (0x58e1c2..0x58e1dd)
for i in 0 .. count-1 (oldest first): rec = record (head + i) mod N
    a = rec.age;  if a >= hdr.life: continue
    C = rec.col + a * rec.col_vel                                     (r, g, b, a)
    f = hdr.fade_start > a ? a * hdr.fade_in : 1 - (a - hdr.fade_start) * hdr.fade_out
    if f < 0: C.a = 0  elif f < 1: C.a *= f
    C = C * elem.colour (+0x40, per channel), each clamped to [0, 255]
    h = a*a*0.5
    S = (rec.scale + a*rec.scale_vel + h*rec.scale_acc) * k * elem.scale (+0x30, per axis)
    ang = -(rec.rot[0] + a*rec.rot[1] + h*rec.rot[2])
    M = diag(S.x, S.y, S.z)
    if rec.flip: M.3x3 = M.3x3 * Flip(rec.flip).3x3                   (0x5bce80, as 4.3)
    if ang != 0: M = M * RotZ(ang)                                    (0x565b50)
    q = rec.pos + a*rec.vel + h*rec.acc
    M.t = pos_is_world ? q : xform(xform(q, L), frame())              (0x58e58c..0x58e629)
    if depth_on: Q = M; Q.t = xform(Q.t, V); Q.t.z -= p.depth; Q = Q * P   (before the billboard)
    M.3x3 = M.3x3 * Vinv.3x3                                          ([0xaa3e38]: screen-aligned)
    depth = depth_on ? 0x5baa40(M, Q, zlen) : 0                       (4.5)
    draw_core(anm, rec.frame, M, C, p.blend (+0x115), depth, keep_depth = 1)   (0x58e747)
```

So each quad is `vertex.xy / 32767 x ANM size x 0.001 x scale% x element scale`, lies in the
screen plane, turned by its own angle (negated, then Vinv), and takes no part or frame scale.
In the default space (FM: 32 blocks, space 255 with mt 0, 1, 3) positions and velocities are
part-local engine space and the sprite **follows the part and its frame every frame**. With
space 7 and mt 0/3/4 (FM: 17 blocks) positions become world at spawn, cone velocities are
rotated by `L` and the frame, the second velocity is in world axes, and the sprite stays where
it was spawned. Game-wide (`survey`): world at spawn and drawn as world 5038, neither 11365;
mixed cases: space 255 with mt 6/9 (transformed twice) 12; space 7 with mt 1, space 6 with mt 0
(local positions taken as world) 13.

### 5.5 Random draws, in order

None per frame unless a sprite is spawned. Per spawned sprite (`rnd`, `rndp`, `irand` as at the
top; a draw is made even when its range is 0):

1. chance: `rand01` if `p.chance < 100` (a rejected sprite still uses up its `r`).
2. flip: one `irand` if `p.flip` is 1, 2 or 3.
3. position: box 3 (x, y, z); cone 3 (ax, ay, az) + radius; arc 1 (ay) + radius; the radius is
   one `rnd` (`+0x100` = 0) or one `rndp` (`+0x101` = 0 or 1), else none; MDL one `irand(n)`
   unless `+0xff` == 2.
4. `+0x104`: speed, acc.
5. cone velocity (shapes 0, 1, 6, 8): ax, ay, az.
6. `+0x106`: dir2 x, y, z, speed2, acc2.
7. `+0x108`: mode 0: base, vel, acc; mode 1: per axis (x then y) base%, vel%, acc%, base, vel,
   acc (12).
8. `+0x10b`: angle, spin, spin acc.
9. `+0x10d`: r, r-vel, g, g-vel, b, b-vel, a, a-vel.

The buffer-full test comes after the chance draw and ends the frame's emission.

### 5.6 Notes

- Spawn age `r*step - 1` spreads one frame's sprites: after the same update's ageing their
  ages are `r*dt/s` (r = n .. 1); with dt = 1 and one sprite per frame the newest is drawn at
  age 1/s in (0.5, 1]. The constant is 1.0, not dt (`0x58d0a8`).
- The rate never changes over time: `rate_acc` only sets the end time and adds the `dt^2` term
  (**INFERRED** meant as rate + acc*t; 4 blocks game-wide, none in FM).
- The interval works through the accumulator: rate 1, interval 2 emits every third frame
  (accumulator 1 -> emit -> -2, -1, 0, 1 -> emit ...).
- The draw uses the ANM frame computed by the last update from the time before its `+dt`.
  Looping ANMs store the **scaled** wrapped time back, so after a wrap the clock mixes scaled
  and unscaled units (FM speeds 1 and 2). A non-looping ANM kills the sprite at its end.
- Death is FIFO: `dead` counts any expired record and removes that many from the head. All
  sprites of a buffer share life and ANM speed, so these are the expired ones.
- With fade on and fade-in 0 the fade-in rate is 1.0: alpha is scaled by the age while age < 1.
- Slam SHO0 part 14 (shape 7, axis (0,1,0) -> (0,-1,0) -> LookAt rows X (1,0,0), Y (0,0,1),
  Z (0,-1,0); `a` = (0, ay, 0)): P = 1.3 x (sin ay, -cos ay, 0), a ring of radius 1.3 in the
  part's engine XY plane with ay uniform in +-180; radial speed 0.15, acc -0.01; dir2 (0,0,-1)
  x -0.05 -> engine vel (0,0,-0.05), acc (0,0,+0.001). Dive SHO0 part 17 is the same with
  R = 4 and rate 3 then 1.
- Dive SHO0 part 11 (shape 2, mt 1): box (0.2, 1, 0.2), radial speed 0; dir2 (0,-1,0) acc
  0.008 +- 0.002 -> engine acc y = +0.008 (FEP y down: the sparks fall); N 7, life 13; rate 1,
  interval 2 from frame 5, rate 0 from frame 11.

