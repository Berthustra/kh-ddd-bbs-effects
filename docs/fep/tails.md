# FEP trails, ribbons and line particles (topic "tails")

STATUS: sections 1-7 written (1.8 Orient still TODO; open points in 7).

DDD Steam exe (KH 2.8), image base 0x140000000, **every address is an RVA**. Labels as in
[README.md](README.md): unmarked = read in the exe at the RVA given; **INFERRED** = follows from
read facts; **GUESS** = a guess. Parameter values quoted are from the seven Flowmotion FEPs
(`tools/ddd_fep_tails_params.py`).

Classes covered (vtable slots in README 11.2):

| id | class | role |
|---|---|---|
| 68 | FecLocus_tail_strategy | records the part's own position every frame; resamples it into N points at fixed arc spacing |
| 79 | FecNull_routegenerator | outputs a zero vec4 (the "route" point = the part origin) |
| 80 | FecShapetail | draws one ANM sprite per Locus point, colour and size lerped head -> tail |
| 105 | FecBindlocus_polygon | textured ribbon (triangle strip) between the two edge arrays of a Wave_bindlocus_strategy, optionally B-spline smoothed, colour lerped head -> tail, UV scroll |
| 106 | FecWave_bindlocus_strategy | records two points of the part (A, B) once per update into N-sample histories; every older sample drifts by its own velocity (constant drift + random "wave") |
| 104 | FecEllipse_line_particle | particles on a sphere spinning about local Y, drawn through diag(axes) (an ellipse), each a line strip back along its arc |
| 100 | FecRadialg_line_particle | particles shot in a cone about local +Y with speed/accel/gravity, each a line strip through its last n positions |

Conventions used below. Matrices are row-vector (`v' = v M`, `0x63ad40`: out = v.x R0 + v.y R1 +
v.z R2 + v.w R3). `mul(A, out, B)` = `0x63ac90(rcx=A, rdx=out, r8=B)` gives out = B x A (B applied
first). `0x63d030` = inverse (identity substituted when the determinant is 0), `0x63d6c0` =
identity, `0x58c6c0` = normalize xyz (zero stays zero, w = 1), `0x63c240` = scale xyz.
`rand01` = `(0x613020() >> 1) * 4.65661e-10` (`[0x7ecf78]`), i.e. [0, 1).
A command's inputs are `{u16 slot[n], u16 work[n]}` at record `+8` (README 6.2); "input k" below
means work[k] of that list, read as `[rec + off + 2n + 2k]` (e.g. `0x5a6dac`, `0x5a2505`).

---

## 1. The dive / rail trail: Null_routegenerator (79) + Locus_tail_strategy (68) + Shapetail (80)

Used by AJC SHO0 part 18 (the dive trail, spawned by part 19's Generateanchorvertex) and both parts
of COM LOC0 (the rail trail). Slot layout is the same in all three parts:

| slot | class | inputs |
|---|---|---|
| 0..3 | Posac, Rot, Scl, Colac | |
| 4 | Lmatrix | Pos, Scl, Rot |
| 5 | Null_routegenerator | (work offset = Lmatrix's, size 0x10) |
| 6 | Locus_tail_strategy | input 0 = slot 4 (Lmatrix), input 1 = slot 5 (route) |
| 7 | Shapetail | input 0 = slot 3 (Colac), input 1 = slot 4 (Lmatrix), input 2 = slot 6 (Locus) |

### 1.1 FecNull_routegenerator (79)

Slot 0 `0x5853d0`: work vec4 = 16 bytes from `0xaa3df0` and return 1. `0xaa3df0` lies in the
zero-filled tail of `.data` (raw size 0x7e600 ends at 0xa37600) and no RIP-relative write to it
exists in `.text` (`tools/ddd_refs.py`: 38 references, all reads); **INFERRED** (0,0,0,0).
Slots 2 `0x5853c0` and 4 `0x5853f0` return 0. Param size 0. So the "route" point is the origin of
the part's own matrix.

### 1.2 FecLocus_tail_strategy (68): parameters (0x26 bytes + 3 read bytes past the size)

| off | type | meaning | read at |
|---|---|---|---|
| 0x00 | vec3 | edge offset A (y, z negated at use) | `0x5a6983..0x5a69dd` |
| 0x0c | f32 | spacing between points (world units of the sample space) | `0x5a6e1c` |
| 0x10 | vec3 | edge offset B (y, z negated) | `0x5a6a08..0x5a6a49` |
| 0x1c | u16 | random spacing min (converted int -> float) | `0x5a6e36` |
| 0x1e | u16 | random spacing max | `0x5a6e3f` |
| 0x20 | s16 | N - 1 (N = points) | `0x5a2154` |
| 0x22 | u16 | history length request H0 | `0x5a215c` |
| 0x24 | u8 | random spacing on | `0x5a6e2c` |
| 0x25 | u8 | build the two edge arrays | `0x5a6be6`, `0x5a634a`, `0x5a6929` |
| 0x26 | u8 | **past size**: space. 0 = part space, else world (frame) space | `0x5a6b9e` |
| 0x27 | u8 | **past size**: re-initialise on loop rewind | `0x5a21d4` |
| 0x28 | u8 | **past size**: 1 = do not carry the stored points with the frame | `0x5a6379` |

Flowmotion values: AJC SHO0 p18: spacing 0.12, N 17, H0 32, everything else 0 (space 0).
COM LOC0 p0 / p1: spacing 0.24 / 0.10, N 17, H0 32, space 1, rest 0.
Game-wide (1500 blocks): `+0x26` = 1 in 914, `+0x28` = 1 in 76, `+0x27` always 0.

### 1.3 The LocusStrategy object

Slot 0 `0x5a2110` allocates 0xd8 bytes from the effect allocator (`0x5a6050 -> 0x5bd400`),
constructs it (`0x5a5fc0`: base LightStrategy ctor `0x5a50d0` sets two identity matrices, vtable
`0x7ed650` = `LocusStrategy` (RTTI), base `0x7ed620` = `LightStrategy`), stores the pointer in work
`+0` (work size 8), then `init` `0x5a6b80` and `reset` `0x5a6d60`. Slot 4 `0x5a21b0` = virtual
destructor (frees the arrays). Slot 1 `0x5a20f0` does nothing (a key never changes the trail).
Slot 2 `0x5a2100` = `update` `0x5a62c0`. Slot 5 `0x5a21d0` = `reset` if `+0x27`, returns 1.

| obj off | meaning |
|---|---|
| +0x08 | matrix M (64 B): the space the points are recorded in (vt[3]) |
| +0x48 | matrix W (64 B): second matrix (vt[3]) |
| +0x88 | s32 N |
| +0x90 | vec4[N] the points P (output; P[0] = newest) |
| +0x98 | f32[N] spacing per gap (gap i = between P[i] and P[i+1]) |
| +0xa0 | vec4[N] edge A points (only `+0x25`) |
| +0xa8 | vec4[N] edge B points (only `+0x25`) |
| +0xb0 | s32 V = points ever reached (valid count, 1..N) |
| +0xb4 | u8 = `+0x25` |
| +0xb8 | f32 progress to the next unreached point (0..1) |
| +0xc0 | vec4[Hc] history ring |
| +0xc8 | s32 Hc = max(H0, (space ? 3 : 2) * N) (`0x5a6bd3..0x5a6be3`) |
| +0xcc | s32 head index (newest history entry) |
| +0xd0 | s32 history entries filled (capped at Hc) |
| +0xd4 | u8 space flag (`+0x26 != 0`) |
| +0xd5 | u8 "first update" flag (ctor sets the word `+0xd4` = 0x100) |

Virtual methods (vtable `0x7ed650`): [0] dtor `0x5a60a0`; [1] `0x5a3660` returns 1; [2]
`0x5a6d40` space kind = `+0xd4 ? 3 : 1`; [3] `0x5a6180` refresh M/W; [4] `0x5a5390` frame delta.

**vt[3] refresh M, W** (`0x5a6180`):
- kind 1 (space 0) -> `0x5a55a0`: M = part local matrix (`part+0x78`), W = part world matrix
  (`part+0x80`), or M when the part has none.
- kind 3 (space 1): F = frame owner's `+0x88` (part `+0x28`), else the element's matrix
  (instance `+0x60`); M = local x F, W = world x F (or M).

**vt[4] frame delta** (`0x5a5390`, out D): D = inv(M_old) W_old inv(W_new) M_new, where `_old`
are M, W before and `_new` after calling vt[3] (every inverse falls back to identity when singular).
With W = M this is inv(M_old) M_new: it moves a point that was recorded under the old matrix to the
same place under the new one.

### 1.4 Locus algorithms (exact)

```
init (0x5a6b80, args N = s16[+0x20] + 1, H0 = u16[+0x22]):
  space = (+0x26 != 0); N = N; V = 0; progress = 0; edges = (+0x25 != 0)
  Hc = max(H0, (space ? 3 : 2) * N); head = 0; filled = 0
  allocate P, A, B (N vec4), spacing (N f32), hist (Hc vec4), all zero

reset (0x5a6d60):               # at spawn, and on loop rewind when +0x27
  route = input 1's work vec4 (Null_routegenerator: 0)
  vt[3]()                        # refresh M, W
  s = (route.x, route.y, route.z, 1) * M     # the part's current position in the sample space
  for i in 0..N-1:
      P[i] = s
      spacing[i] = +0x24 ? min + (max - min) * rand01 : f32[+0x0c]   # min/max = u16 +0x1c/+0x1e
  head = (Hc - 1) % Hc; hist[head] = P[0]; V = 1; filled = 1   # (0x5a6f0b..0x5a6f6c)

update (0x5a62c0), every frame (update list):
  edges = (+0x25 != 0)
  if (part.+0x91 & 7) in {3, 4} or u8[+0x28] == 1:
      vt[3]()                    # follow the matrices, do not move the stored points
  else:
      D = vt[4]()                # frame delta
      for every P[i] and every hist[j] (all Hc entries): p.xyz = (p.xyz, 1) * D
  if ctx flag bit 0 (time stopped) or instance byte +0x0a or instance dt (+0x13c) == 0: return
  if first (+0xd5):              # (0x5a64c1..0x5a6569)
      s = (route, 1) * M; P[0..N-1] = s; hist[head] = s; first = 0
  filled += 1
  head = (head - 1 + Hc) % Hc
  hist[head] = (route, 1) * M ;  P[0] = hist[head]          # newest point = current position
  nxt = (head + 1) % Hc
  if filled > Hc: filled = Hc; end = (nxt + Hc - 1) % Hc (= head)   else: end = 0
  # walk the history polyline from the newest point, dropping a point every spacing[i-1]
  prev = &hist[head]; k = nxt; i = 1
  while i < N:
      d = spacing[i - 1]
      loop:
          seg = |hist[k] - *prev|          # (0x5a6700..0x5a676c; sqrt, 0.001 if negative)
          if seg >= d: break
          d -= seg; prev = &hist[k]; k = (k + 1) % Hc
          if V < N: progress = (spacing[i-1] - d) / spacing[i-1]
          if k == end: goto done          # history exhausted: P[i..] keep old values
      dir = normalize(hist[k] - *prev)
      P[i] = *prev + dir * d (w = 1)      # (0x5a67cf..0x5a6886)
      if i >= V: V += 1; progress = 1.0
      prev = &P[i]                         # next distance is measured from the new point
      if k == end: goto done
      i += 1
  done:
  if edges:                                # (0x5a6929..0x5a6b33)
      R = M with the translation row zeroed (row 3 = (0,0,0,1))
      a = (x, -y, -z, 1) * R  for vec3 +0x00 ;  b = same for vec3 +0x10
      for i in 0..N-1: A[i] = P[i] + a ;  B[i] = P[i] + b
```

So the trail is a **history of the part's own origin** (route point 0 through the part's local
matrix, or local x frame in world space), resampled at constant arc length: P[0] is where the part
is now, P[i] lies sum(spacing[0..i-1]) behind it along the path it travelled. V counts the points
the path has reached so far (it starts at 1 and grows as the part moves), so a trail grows out of
the part and stays at full length; points past the recorded path keep their previous position.
There is no per-sample lifetime: a point exists while the path behind the part is long enough, and
the whole trail dies with its part. With `+0x28 = 0` and space 0 the recorded points are moved by
every change of the part's local/world matrix (vt[4]); in world space (space 1) by the change of
local x frame, i.e. the trail does **not** stay in the world when the frame moves, unless `+0x28`
is 1 or Lmatrix byte 1 & 7 is 3 or 4.

### 1.5 FecShapetail (80): parameters (0x48 bytes + 1 read byte past the size)

| off | type | meaning | read at |
|---|---|---|---|
| 0x00 | vec4 | colour at the head (P[0]), RGBA, 128 = 1.0 | `0x5a29a3..0x5a29c2` |
| 0x10 | vec4 | colour at the tail | `0x5a2a09` |
| 0x20 | vec3 | orient vector, used only when `+0x47` | `0x5a2e4b`, `0x5a32ca` |
| 0x2c | s32 | ANM index (section 4); < 0 = draw nothing | `0x5a23a6`, `0x5a28cc` |
| 0x30 | f32 | animation speed (ANM frames per element frame) | `0x5a240b` |
| 0x34 | f32 | size % at the head | `0x5a2ace` |
| 0x38 | f32 | size % at the tail | `0x5a2aa4` |
| 0x3c | f32 | random size amount (size x (1 - rand01 x this)) | `0x5a2290` |
| 0x40 | f32 | no reader in the class (1.0 in AJC, 0 in COM) | - |
| 0x44 | u8 | random start time of the animation | `0x5a2727` |
| 0x45 | u8 | random size per point on | `0x5a2268` |
| 0x46 | u8 | random rotation per point on (0..180 deg) | `0x5a2306` |
| 0x47 | u8 | orient on | `0x5a2aca`, `0x5a2e21`, `0x5a327a` |
| 0x48 | u8 | **past size**: blend. 0 -> 4 = each ANM quad's own blend code; 1..3 -> `0x5c02d0` table `0x7ed8e0` = 2, 3, 1 | `0x5a2abd..0x5a2ac7` |

Flowmotion: AJC SHO0 p18: head (160,110,120,128), tail (64,120,32,0), ANM 7 = local 0x9803AC87
(19 keys x 2 frames, 32 quads, not looping, texture F0000015, quad blend 0xA0600, ANM `+0x38` =
49.125), speed 1, size 5% -> 30%, random rotation on, blend byte 0. COM LOC0 p0: head (96,64,128,24),
tail (32,32,128,0), ANM 0 = resident F00000AB (1 key, looping, `+0x38` = 62), speed 2, size 20% ->
10%, orient on with (0,0,-0.5), blend byte 1 (-> blend index 2). COM LOC0 p1: head (128,64,128,64),
size 5% -> 5%, otherwise as p0, spacing 0.1. Game-wide (1562 blocks) `+0x48` = 0:873, 1:515,
3:132, 2:42.

Work (8 bytes) -> a 0x20-byte state: `+0x00` f32[N] size factor per point, `+0x08` f32[N] rotation
per point (degrees), `+0x10` f32 animation clock, `+0x14` s32 key search start (always 0), `+0x18`
s32 current key (-1 = animation over, nothing drawn). N = the Locus object's N (input 2).

### 1.6 Shapetail algorithms (exact)

```
slot 0 (0x5a24c0): N = Locus.N; size[i] = 1.0; rot[i] = 0
    if +0x44 and the ANM exists: clock = rand01 * (ANM.length - 1)     # ANM +0x14
slot 1 key (0x5a2200), new block:
    size[i] = +0x45 ? 1 - rand01 * f32[+0x3c] : 1.0
    rot[i]  = +0x46 ? rand01 * 180 : 0
slot 2 update (0x5a2390):
    clock += instance dt (+0x13c); f = clock * speed
    if f >= ANM.length:
        if ANM flags (+0x10) & 0x80: f = float(int(f) % int(length)); clock = f / speed
        else: key = -1; return
    key = first k with f < sum(duration[0..k])     # ANM keys {flags, frame, duration}, from 0
slot 3 draw (0x5a2850): see 1.7
slot 4 (0x5a27d0): free the arrays; slot 5 (0x5a34a0): return 1 (loop changes nothing)
```

### 1.7 Shapetail draw (exact)

```
if ANM index < 0, record missing, slot -1, or key < 0: draw nothing
C = input 0 (Colac output, 0..255)
head = C * colHead / 128 ; tail = C * colTail / 128            # per component (0x63b0c0, 0.0078125)
V = Locus.V (points reached)
dcol = (tail - head) / V
s = sizeHead * 0.01 ; ds = (sizeTail - sizeHead) / V * 0.01
blend = u8[+0x48] ? table[u8[+0x48]] : 4
lm1 = Lmatrix(input 1) param byte 1
if +0x47: zs = |row 2 of F| (F = frame owner +0x88 or instance +0x60); else zs unused
if lm1 == 0:
    D = 0x5b2670(ctx, part, part local, Lmatrix byte 0, byte 1, byte 2, 1.0) if Locus kind 1
        identity if Locus kind 3
elif lm1 == 1:
    D = F if Locus kind 1, identity if kind 3
else: draw nothing
for i in 0..V-1:
    head = clamp(head, 0, 255)                              # stored back (0x5a2c80..0x5a2cf3)
    col = clamp(instance colour (+0x40) * head, 0, 255)     # 0x63b0c0, per component
    k = s * size[i] * (ANM.f32[+0x38] * 0.1)
    if lm1 == 0:
        T = diag(k, k, k, 1) * RotZ(rot[i]); T.row3 = P[i]; T = T x D     # 0x565b50, 0x63ac90
    else:  # camera-facing
        T = diag(k*Sx, k*Sy, k*Sz, 1) * RotZ(rot[i])   # S = instance scale (+0x30)
        T.row3 = (P[i], 1) * D
        T.3x3 = T.3x3 x CAM.3x3                          # 0x63a950, CAM = *[0xaa3e38]
    zoff = 0
    if +0x47: (orient helper 0x5b39a0 / 0x5a3420, then zoff = 0x5baa40(T, out, zs)) - see 1.8
    0x55a360(ANM table ctx+0xa8, ANM slot, key, T, col, blend, zoff, 1)  # ANM sprite draw
    head += dcol ; s += ds
```

RotZ(a) (`0x565b50`): a in degrees (x 0.711111 x 0.0245437 = radians), T' = T x Rz, Rz rows
(c, s, 0), (-s, c, 0), (0, 0, 1) (commands.md 6.2). The ANM draw `0x559eb0` multiplies each
quad vertex colour by `col >> 7` (`0x55a184..0x55a1ab`) and, with blend 4, sets the blend from each
quad's code through `0x5c02a0` (0x70600 -> 1, 0xA0600 -> 2, 0xA0602 -> 3); otherwise from
`0x5c02d0(blend)` once (`0x55a0b4..0x55a0d4`).

So a Shapetail trail is **a chain of V camera-facing (or frame-aligned) ANM sprites**, one per
Locus point, colour and size lerped linearly from head to tail over the reached points (the last
reached point gets head + (V-1)/V of the way, never exactly the tail value), all showing the same
animation key. The dive trail (AJC p18): 17 sprites 0.12 apart (2 units of trail), size
4.9 x 0.05 -> 4.9 x 0.3 (ANM units; the ANM quads span +-16384, INFERRED 1.0), colour
(160,110,120,128) -> (64,120,32,0) x Colac, each randomly rotated about the view axis.

### 1.8 Orient (`+0x47`, COM LOC0 only) TODO

---

## 2. Bindlocus_polygon (105) + Wave_bindlocus_strategy (106)

Used by AJC CAL0, AKD CAL0, COM LOC1 (2 parts), COM LOC2, BOS LOC1, COM FSH0, COM SBT0 (2 parts)
and the Keyblade trail W_010 LOC0. Slot layout (W_010, AJC CAL0; COM LOC1 p0 shifted by one):

| slot | class | inputs |
|---|---|---|
| 0..3 | Pos, Rot, Scl, Colac (or Posac/Rotmv/Sclac ...) | |
| 4 | Lmatrix | Pos, Scl, Rot |
| 5 | Wave_bindlocus_strategy | input 0 = Lmatrix (not read: the class uses the part matrices directly) |
| 6 | Bindlocus_polygon | input 0 = Colac, input 1 = Lmatrix (its param bytes), input 2 = the strategy |

Both update in slot order (strategy first, then the polygon rebuilds its vertices from the
fresh samples), the polygon draws.

### 2.1 Wave_bindlocus_strategy (106): parameters (0x4c bytes)

| off | type | meaning | read at |
|---|---|---|---|
| 0x00 | vec3 | edge point A in the part's space (y, z negated at use) | `0x5a195f`, reset `0x5a0ae4` |
| 0x0c | s16 | N = samples per edge (history length, one sample per update) | `0x5a1cd0`, `0x5a12cf`, `0x5a14b2` |
| 0x0e | u8 | wave re-target period P (updates) | `0x5a1a4d` |
| 0x0f | u8 | 1 = do not carry the samples with the frame | `0x5a10fd` |
| 0x10 | vec3 | edge point B (y, z negated) | `0x5a19ec`, reset `0x5a0d8e` |
| 0x1c | f32 | no reader | - |
| 0x20 | vec3 | drift per update, **not** rotated (x, -y, -z) | `0x5a1c2f..0x5a1c56`, reset `0x5a0a8e` |
| 0x30 | vec3 | wave follow rate per axis (0..1 per update) | `0x5a1b27`, `0x5a1b46`, `0x5a1b6b` |
| 0x40 | vec3 | wave amplitude per axis (target uniform in [-amp, amp]) | `0x5a1a59`, `0x5a1a8a`, `0x5a1abb` |

Flowmotion values (`ddd_fep_tails_params.py --ids 106`): drift 0 everywhere. AJC CAL0 / AKD CAL0:
A (0,-1.3,0), B (0,1.3,0), N 16, P 6, carry off (`+0x0f` = 1), rate (1,0,1), amp (0.03,0,0.03).
W_010: A (0,0,1.05), B (0,0,-0.027), N 6, P 10, carry on, rate 0, amp 0 (no wave). Others: N 6..10,
rate 0 or (0.15,0,0.15), amp 0 or (1,0,1). Bytes past 0x4c: none read.

### 2.2 The WaveStrategy object (0xd0 bytes)

Slot 0 `0x5a82c0`: allocate 0xd0 (`0x5a0690` -> `0x5bd400`), ctor `0x5a0570` (two identity
matrices, rest 0), work `+0` = pointer (work size 8), then `init` `0x5a1cb0(obj, params)`; returns
0 (spawn aborted) if any allocation fails. Slot 1 `0x5a82a0` returns 0 (a key does nothing;
the update reads the current params pointer every frame, so a key's new values do apply).
Slot 2 `0x5a82b0` = `update` `0x5a1060(obj, ctx, params)`. Slot 4 `0x5a8320` frees the four
arrays (`0x5a05e0`) and the object. Slot 5 `0x5a8370` returns 1 (loop rewind changes nothing).
Slot 8 `0x5a8350`: section 3.

| obj off | meaning |
|---|---|
| +0x00 | matrix M = part local (`part+0x78`) x F |
| +0x40 | matrix W = part world (`part+0x80`) x F, or M when the part has no world matrix |
| +0x80 | vec3 wave offset `cur` (ctor (0,0,0,1); init 0) |
| +0x90 | vec3 wave target `tgt` |
| +0xa0 | vec4[N] A (edge A samples, [0] = newest) |
| +0xa8 | vec4[N] VA (velocity of each A sample) |
| +0xb0 | vec4[N] B |
| +0xb8 | vec4[N] VB |
| +0xc0 | u8 reset request (init sets 1; slot 8 sets 1) |
| +0xc4 | s32 re-target countdown (init 1) |
| +0xc8 | s32 N |

F = the frame: owner part's `+0x88` (part `+0x28`), else the element instance matrix `+0x60`
(`0x5a0940`). Unlike Locus there is no space switch: **always local x frame (world space)**.

### 2.3 Wave algorithms (exact)

```
refresh (0x5a0940):  F = part.owner ? owner+0x88 : inst+0x60
                     M = part.local x F            # local applied first (0x63ac90)
                     W = part.world ? part.world x F : M

init (0x5a1cb0):     reset_req = 1; allocate A, VA, B, VB (N vec4, zero); cur = tgt = 0;
                     countdown = 1; N = s16[+0x0c]

reset (0x5a0a60):    # run by the first update after init / after slot 8
  d  = (drift.x, -drift.y, -drift.z, 0)
  refresh()
  a0 = (pA.x, -pA.y, -pA.z, 1) * M
  for i in 0..N-1:  A[i] = a0 + i*d (w = 1);  VA[i] = d
  b0 = (pB.x, -pB.y, -pB.z, 1) * M
  for i in 0..N-1:  B[i] = b0 + i*d;  VB[i] = d

update (0x5a1060, every frame from the update list):
  if reset_req: reset_req = 0; reset(); return
  if (part.lm_byte1 & 7) in {3, 4} or u8[+0x0f] == 1:      # part+0x80 -> +0x91
      refresh()                                            # samples stay where recorded
  else:
      D = inv(M_old) W_old inv(W_new) M_new                # refresh() in between; inverses
                                                           # fall back to identity when singular
      for i: A[i].xyz = (A[i].xyz, 1) * D ; B[i] likewise
      R = D with row 3 = (0,0,0,1)
      for i: VA[i].xyz = (VA[i].xyz, 1) * R ; VB[i] likewise
  if inst.dt (+0x13c) == 0: return
  for i = N-1 down to 1:                                   # age every sample by one update
      A[i] = A[i-1] + VA[i-1] (w 1); VA[i] = VA[i-1]
      B[i] = B[i-1] + VB[i-1];       VB[i] = VB[i-1]
  A[0] = (pA.x, -pA.y, -pA.z, 1) * M                       # new head samples
  B[0] = (pB.x, -pB.y, -pB.z, 1) * M
  countdown -= 1
  if countdown == 0:
      countdown = u8[+0x0e]                                # (0 -> next wrap after 2^32 updates)
      tgt.k = (2*rand01 - 1) * amp.k   for k = x, y, z     # three rand01 calls, x first
  cur.k += (tgt.k - cur.k) * rate.k
  w = (cur, 1) * M - (0, 0, 0, 1) * M                      # cur rotated (and scaled) by M
  VA[0] = VB[0] = (w.x + drift.x, w.y - drift.y, w.z - drift.z, 0)
```

So each update pushes the current positions of the two part points into the histories; an
older sample moves by the velocity it was born with (drift plus the smoothed random wave of that
moment), so the ribbon "waves" and drifts behind the part. N samples = the last N updates.

**Carry** (`D`): M and W are local x F and world x F with the same new F, and for a part without a
parent world = local (commands.md 7.2), so W = M and **D = identity** whatever the frame does
(the formula reduces to inv(M_old) M_old inv(M_new) M_new; the other multiplication order also
gives identity). Hence for every Flowmotion part (no parents) the samples stay in world space
whatever `+0x0f` says; D differs from identity only for child parts whose world matrix includes a
parent (Lmatrix modes 0, 3, 4).

**Update rate**: the update is called once per pass-1 update, i.e. once per rendered frame with
dt != 0 (README 7.1), and nothing here is scaled by dt: on PC (60 fps, dt 0.5) a sample is taken
every 1/60 s, so N = 6 spans 5 game frames. **INFERRED** from the code; see 7.

### 2.4 Bindlocus_polygon (105): parameters (0x56 bytes)

| off | type | meaning | read at |
|---|---|---|---|
| 0x00 | vec4 | colour at the head (newest sample), 128 = x1 of the input colour | `0x59d6a8` |
| 0x10 | vec4 | colour at the tail | `0x59d731` |
| 0x20 | vec3 | depth-offset vector (`+0x55`; y, z negated by `0x5b39a0`) | `0x59e1fe` |
| 0x2c | s32 | texture index (FEP section 2); < 0 with `+0x48` = draw nothing | `0x59d41c`, `0x59ded7` |
| 0x30 | f32 | U scroll, texels per FEP frame (U -= dt x this) | `0x59d466` |
| 0x34 | f32 | V scroll (V -= dt x this) | `0x59d476` |
| 0x38 | f32 | head fade length in % of the points (`+0x50`) | `0x59d835` |
| 0x3c | f32 | no reader (1.0 in all Flowmotion blocks) | - |
| 0x40 | u16 | U offset (texels) | `0x59d32d`, `0x59e553` |
| 0x42 | u16 | V offset (texels) | `0x59d343`, `0x59e598` |
| 0x44 | u16 | U extent (texels) | `0x59d8ca`, `0x59d975` |
| 0x46 | u16 | V extent (texels) | `0x59d90a`, `0x59d945` |
| 0x48 | u8 | textured | `0x59d412`, `0x59d87b`, `0x59df2c` |
| 0x49 | u8 | random U start (U = Uoff + rand x (Uext - Uoff), `0x58c630` **INFERRED** rand01 x arg) | `0x59e534` |
| 0x4a | u8 | random V start | `0x59e574` |
| 0x4b | u8 | sampler: 1 -> `0x5c0710(.., 5, 1, 0, 0)` (as every effect draw), 0 -> (0, 0, 0, 0) | `0x59dfd7` |
| 0x4c | u8 | 0: U across the ribbon, V along it; 1: swapped | `0x59d89b` |
| 0x4d | u8 | flip U (across) / U direction (swapped) | `0x59d8c6`, `0x59d96d` |
| 0x4e | u8 | flip V | `0x59d8f7`, `0x59d941` |
| 0x4f | u8 | 0 = reset viewport/scissor around the draw (`0x5e30f0` / `0x66f890`, draw-models.md) | `0x59e034`, `0x59e42b` |
| 0x50 | u8 | head fade on | `0x59d820` |
| 0x51 | u8 | blend index -> `0x5c02d0` (table `0x7ed8e0`: 0->1, 1->2, 2->3, 3->1) | `0x59e04c` |
| 0x52 | u8 | smooth: resample both edges with a spline object | `0x59dd5c`, `0x59d5bf`, `0x59e0bd` |
| 0x53 | u8 | spline kind: 0 = "cubic" (0x28-byte object, vtable `0x7ed598`), 1 = uniform cubic B-spline (0x30, vtable `0x7ed568`); other = spawn fails | `0x59dd62` |
| 0x54 | u8 | spline subdivision S | `0x59ddbf` |
| 0x55 | u8 | depth offset on | `0x59e154`, `0x59e3d4` |

Flowmotion (20 blocks): all textured, blend 1 (-> index 2, **additive weighted by alpha**,
render-state.md 2), smooth on with the B-spline (S 30..65), U offset 0, U scroll 0; V scroll 0,
-2, -3 or -5; head fade on except AJC CAL0; depth offset on except AJC/AKD CAL0. Textures are
resident (F000001D 128x64 for most, F0000009 128x128 for W_010, F0000006).

Work (0x20 bytes): `+0x00` handle of the vertex buffer (48 x count bytes from the effect
allocator), `+0x04` handle of the spline object, `+0x10` f32 U, `+0x14` f32 V (texels), `+0x18`,
`+0x1c` f32 U/V extent (written, not read by the update).

### 2.5 Bindlocus algorithms (exact)

```
slot 0 (0x59dcf0): S = input 2's object (part work + 2*work[2], the WaveStrategy)
    uv_init(randomize = 1)                                          (0x59e4a0, below)
    if +0x52: spl = new (+0x53 == 0 ? Cubic : BSpline); spl.vt[1](S.N, u8 +0x54); count = spl.vt[3]()
    else:     count = S.N
    vbuf = alloc(48 * count), zeroed                                 # 2 vertices x 24 bytes per point
uv_init (0x59e4a0, slot 0 with randomize 1; slot 1 0x59d2a0 = the same with randomize 0, run
         for every key block): only when textured and the texture exists
    Uext = u16 +0x44; Vext = u16 +0x46
    U = +0x49 ? (randomize ? Uoff + rnd(Uext - Uoff) : unchanged) : Uoff
    V = +0x4a ? (randomize ? Voff + rnd(Vext - Voff) : unchanged) : Voff
slot 2 update (0x59d360):
    if textured and the texture exists:
        U -= dt * f32 +0x30 ; V -= dt * f32 +0x34                      # dt = inst +0x13c
        TW, TH = texture width, height (texture object [+8] +0x1c, +0x20, ints)
        wrap U into [0, TW) and V into [0, TH) by repeated -/+ TW (TH)
    if +0x52: spl.vt[2](S.A, S.B, 1, 1); PA = spl+0x18; PB = spl+0x20; count = spl.vt[3]()
    else:     PA = S.A; PB = S.B; count = S.N
    G = input0.vec4 (Colac, 0..255) * inst colour (+0x40) * (1/128)    # per component
    head = clamp(colHead * G, 0, 255) ; tail = clamp(colTail * G, 0, 255)
    dcol = (tail - head) / count
    fstep = 0; f = 1
    if +0x50:
        k = int(f32 +0x38 * 0.01 * (count - 1) + 1.0)                 # truncation
        if k > 1: fstep = 1/k; f = 0
    if textured:
        if +0x4c == 0:                       # U across, V along
            uA, uB = +0x4d ? ((U + Uext)/TW, U/TW) : (U/TW, (U + Uext)/TW);  du = 0
            v0, dv = +0x4e ? (V/TH, +Vext/TH/count) : ((V + Vext)/TH, -Vext/TH/count)
            vA = vB = v0
        else:                                # swapped
            vB, vA = +0x4e ? ((V + Vext)/TH, V/TH) : (V/TH, (V + Vext)/TH);  dv = 0
            u0, du = +0x4d ? (U/TW, +Uext/TW/count) : ((U + Uext)/TW, -Uext/TW/count)
            uA = uB = u0
    c = head
    for i in 0..count-1:
        c = clamp(c, 0, 255)                                         # stored back
        a = c.a;  if f < 1: a = min(255, a * f)
        f += fstep
        rgba = int(c.r) | int(c.g) << 8 | int(c.b) << 16 | int(a) << 24     # cvttss2si
        textured:   vtx[2i]   = {f32 uA, f32 vA, u32 rgba, f32 PA[i].xyz}    # 24 bytes
                    vtx[2i+1] = {f32 uB, f32 vB, u32 rgba, f32 PB[i].xyz}
                    uA += du; vA += dv; uB += du; vB += dv
        untextured: vtx[2i] = {rgba, PA[i].xyz}; vtx[2i+1] = {rgba, PB[i].xyz}  # 16 bytes
        c += dcol
slot 3 draw (0x59deb0): 2.7.   slot 4 (0x59de50): free vbuf and spline.   slot 5 (0x59e5d0): 1.
```

Point 0 is the newest sample (at the part now), so the head colour and the head fade sit at the
part and the colour runs to `head + (count-1)/count (tail - head)` at the oldest sample. V runs
from `(Voff + Vext)` at the head down by Vext/count per point (unflipped).

### 2.6 The spline objects (both edges resampled the same way)

Object: `+0x08` s32 N (control points), `+0x0c` s32 S, `+0x10` s32 count, `+0x18` vec4[count]
out A, `+0x20` vec4[count] out B, B-spline `+0x28` f32 step. vt[3] = `0x5a0560` returns count.

```
BSpline.init (vt[1] 0x59ef10, N, S):
    step = float(N + 2) / float(S)
    t = -1.0; count = 0
    do { t += step; count += 1 } while (float(N) >= t)               # float accumulation
BSpline.eval (vt[2] 0x59e730, P = A or B; done for A into +0x18 and B into +0x20 in one loop):
    t = -1.0
    repeat count times (exactly: while float(N) >= t after each step, first pass always):
        out(t) = sum over j = -2 .. N+1 of b(t - j) * P[clamp(j, 0, N-1)]   (xyz; w not written)
        t += step
    b(x): x = |x|;  x < 1: (3x^3 - 6x^2 + 4)/6 ;  x < 2: (2 - x)^3 / 6 ;  else 0
Cubic.init (vt[1] 0x5a04d0, N, S): fails if N > 32; count = (N - 1) * S + 1.  (vt[2] = 0x5a04b0
    -> 0x59f1b0 not decoded: no Flowmotion block uses kind 0.)
```

The B-spline (end points duplicated twice) passes exactly through P[0] at t = -1 and P[N-1] at
t = N; in between it smooths the samples. W_010: N 6, S 32 -> step 0.25, count 29 (58 vertices);
AJC CAL0: N 16, S 64 -> step 0.28125, count 61.

### 2.7 Bindlocus draw (`0x59deb0`)

```
if textured and (index < 0 or no texture): draw nothing
state = 0x5e1960(group); frag = 0x5e19a0(); cmd = 0x5e1f20()
if textured: preset +0xdc (textured, K = 0x80808080, mode 3); bind texture 0x5c0710(.., id,
             +0x4b ? (5, 1) : (0, 0), 0, 0); stride = 0x30 per point
else:        preset +0xf8 (untextured, mode 2); stride = 0x20
if +0x4f == 0: viewport/scissor reset (0x5e30f0)
blend = table[u8 +0x51]; frag+0x80 |= 1; 0x5e2300(frag, blend); flush 0x5e2ea0
copy count*stride bytes of vbuf into per-frame vertex memory (0x5e1b80)
if +0x55:                                                    # depth offset, as draw-sprites 4.5
    Mz = 0x5b2190(ctx, part, identity, lm b0, lm b1, lm b2, 1.0)      # lm = input 1's params
    Q  = 0x5b39a0(ctx, part, part local, lm b1, 1.0, space 3, &p+0x20, (1,1,1,1))
    depth = 0x5baa40(Mz, Q, |frame row 2|); 0x5e3090(state, depth, P.m22)
shader [0xaaa950] (0x601480); format (vt +0x10)(3 textured / 2 untextured); vertex ptr (0x6007c0)
world matrix = identity (0x601b00)                           # vertices are already in world space
0x5e33f0(.., prim 6, first 0, 2*count)                       # one triangle strip A0 B0 A1 B1 ...
if +0x55: 0x5e3090(state, 0, 0);  if +0x4f == 0: restore viewport (0x66f890)
submit 0x5e0d30
```

Colour reaches the screen as for the model draws (render-state.md 3): the vertex colour bytes
x K/128 (= 1) x 255/128 (combiner B = 0x202 for the textured preset), i.e. **byte 128 = 1.0**,
times the texture. Depth write and cull are not set by this class (render-state.md: the effect
draws set blend, depth write and cull; this one sets only blend) - see 7.

## 3. Who calls vtable slot 8 (+0x40)

Nobody, in this exe (**INFERRED** dead code):
- Only Wave_bindlocus_strategy (`0x5a8350`) and Wave_tail_strategy (`0x5a8440`) override it;
  each function is referenced only by its vtable (`0x7ecde0`, `0x7ecc00`) and has no direct call
  or jump (`tools/ddd_fep_tails_slot8.py`).
- None of the 16 code sites that reach the command table `0x9e7450` calls `[reg + 0x40]` (same
  tool). The only readers of a part's byte `+0x1b` (slot-8 list count) are `0x5815d3` and
  `0x59c931`, neither a dispatcher, and no `call [reg + 0x40]` anywhere in `.text` is preceded by a
  read of `+0x1b` / `+0x44` of a part (`tools/ddd_fep_tails_slot8b.py`: all hits are game code
  with unrelated stack offsets). The default stub `0x5aac50` has no RIP-relative reference.

What it would do: `slot8(cls, ctx, params, r9, work, u32 flags)`: if `flags & 2`, set the
strategy's reset request (Wave_bindlocus `obj+0xc0 = 1` via `0x5a1e10`, Wave_tail `obj+0xec = 1`
via `0x5a8290`); the next update then re-lays the ribbon at the part's current points (2.3
`reset`) instead of recording. A port can leave it out.

## 4. FecEllipse_line_particle (104) (ASK MLC0 part 15)

Inputs: 0 = colour command (Colac), 1 = Lmatrix. Vtable slots: 0 `0x560070`, 1 `0x55fa10`
(returns 0), 2 `0x55fa20`, 3 `0x560130`, 4 `0x560100` (free), 5 `0x560980` (returns 1).

### 4.1 Parameters (0x4d bytes)

| off | type | meaning | read at | ASK MLC0 |
|---|---|---|---|---|
| 0x00 | vec3 | ellipse axes: draw scale diag(x, y, z) | `0x5601b9..0x5601da` | (0.18, 0.01, 0.18) |
| 0x0c | f32 | angular speed about local Y, deg per update | `0x55fd58` | -10 |
| 0x10 | vec4 | colour at the head (128 = 1) | `0x560413`, `0x560534` | (128, 96, 64, 128) |
| 0x20 | vec4 | colour at the tail | `0x560419` | (128, 64, 64, 0) |
| 0x30 | f32 | angular speed random +- | `0x55fd1c` | 3 |
| 0x34 | f32 | angular acceleration (deg per update, composed each update) | `0x55fde4` | 0 |
| 0x38 | f32 | angular acceleration random +- | `0x55fdb1` | 0 |
| 0x3c | f32 | start radius (x rand01) | `0x55fcc9` | 10 |
| 0x40 | f32 | emission rate | `0x55fa6c` | 1 |
| 0x44 | u16 | max particles | `0x5600a6` | 100 |
| 0x46 | u16 | life (updates); 0 aborts the spawn | `0x55fe85`, `0x5600d2` | 18 |
| 0x48 | u16 | life random +- | `0x55fe53` | 0 |
| 0x4a | u8 | points per line n | `0x5600b0`, `0x5603b3` | 3 |
| 0x4b | u8 | emission interval (updates) | `0x55faa8` | 0 |
| 0x4c | u8 | blend: 0 -> index 1, 1 -> 2 (additive), 2 -> 3, other -> 1 (`0x55f830` then table `0x7ed8e0`) | `0x5602b8` | 1 |

### 4.2 State

Work: `+0` pool (0x18-byte object `0x5639f0`; `+0` particle array, `+8` max, `+0xc` vertex bytes
= 16 x max x n), `+8` f32 emission accumulator (0). Particle (0xe0 bytes):
`+0x00` vec4 pos (part-local, before the axes), `+0x10` mat4 Rs (rotation per update), `+0x50` mat4
Ra (acceleration rotation), `+0x90` mat4 = Ra (copy used by the draw), `+0xd0` u16 life left,
`+0xd2` u16 fade start = life_param >> 1, `+0xd4` f32 fade, `+0xd8` f32 fade step = 2 / life_param,
`+0xdc` u8 alive.

### 4.3 Update (`0x55fa20`, exact)

```
if ctx flag bit 0 or inst byte +0x0a or inst dt == 0: return
emit(rate = f32 +0x40, interval = u8 +0x4b, acc = work+8)          # shared rule, 4.5
for each emitted particle: take the first free slot at or after the running index (none: skip)
    d = (2r-1, 2r-1, 2r-1) (three rand01, x first); L = |d| (0.001 if L^2 < 0)
    d = L == 0 ? 0 : d / L          (w keeps the slot's old +0x0c)
    pos = d * (rand01 * f32 +0x3c)  (all four components)
    w  = wrap_pi((f32 +0x0c + (2r-1) * f32 +0x30) * pi/180);  Rs = I x RotY(-w in degrees)
    al = wrap_pi((f32 +0x34 + (2r-1) * f32 +0x38) * pi/180); Ra = I x RotY(-al deg); +0x90 = Ra
    life = int((2r-1) * u16 +0x48) + u16 +0x46    (u16, truncation)
    halfl = u16 +0x46 >> 1; fade = 1.0; fstep = 2.0 / u16 +0x46; alive = 1
for every alive particle (in slot order):
    life -= 1; if life == 0: alive = 0; continue
    if life < halfl: fade = max(0, fade - fstep)
    pos = (pos.xyz, 1) * Rs
    Rs  = Rs x Ra                                   # Rs applied first
```
`wrap_pi` adds/subtracts 2 pi until the value is in [-pi, pi]. RotY (`0x55f3e0`) rows
(c,0,-s),(0,1,0),(s,0,c) with the angle in degrees (commands.md 6.2). The identity source is
`0x9e8230` (identity in the file, no writes seen). Note the angle passed is **minus** the
parameter (division by -0.0174533, `[0x7b1208]`).

### 4.4 Draw (`0x560130`, exact)

```
E = diag(axes.x, axes.y, axes.z, 1)
Wm = 0x5b2190(ctx, part, E x part.local, lm b0, lm b1, lm b2, 1.0)    # draw-sprites 4.4, f = 1
untextured preset +0xf8 (K 0x80808080); blend as 4.1 (0x55f890(state, 0, idx))
C = clamp(input0.vec4 * inst colour (+0x40), 0, 255) / 255            # [0x9e82e8] = 1/255 x4
n = u8 +0x4a; dcol = (colHead - colTail) / (128 n)
v = 0 (vertex index); lines = 0
for each particle slot (alive only):
    am = C.a * fade
    Q = inv(Rs); T = identity; c = colHead / 128
    for k in 0..n-1:
        vertex[v].pos  = (pos.xyz, 1) * T                            # still part-local
        T = T x Q ; Q = Q x Ra(+0x90)
        vertex[v].rgba = bytes of min(1, max(0, (c.r C.r, c.g C.g, c.b C.b, c.a am))) * 255
        c -= dcol; v += 1
    lines += 1
if lines: shader [0xaaa940], format 2 ({u32 rgba, f32 x, y, z}, 16 bytes), world matrix = Wm,
          0x5e33e0(.., prim 0xff, 0, n per primitive, lines primitives)
restore 0x55f850 (frag +0x1c = 5, +0x48 = 4, dirty 0x24), submit
```
Primitive 0xff becomes 4 (2 when the pipeline type is 2) in `0x671980` and is drawn `lines` times
with n consecutive vertices each: **INFERRED a line strip per particle** (the numbering of 2 / 4 was
not decoded). So each particle is a short arc: its position now and k updates back along its spin
(exactly back when Ra = identity). The whole thing is drawn through E: a sphere of radius 10 x rand
squashed to an ellipse 1.8 x 0.1 x 1.8 (ASK MLC0), spinning at 10 +- 3 deg per update.

### 4.5 Emission rule (shared by 104 and 100: `0x55fa6c..0x55fb1a`, `0x565126..0x56518a`)

```
x = dt * rate + acc
if x < 1: count = 0; acc = x
else:
    if dt < 1: x = rate                        # (sic) the accumulator is dropped
    count = int(x); acc = x - count - rate * interval
```
With PC dt = 0.5, rate 1, interval 12 (COM BSS0): emits 1, then acc = -12, +0.5 per update ->
next emission 26 updates (13 FEP frames) later. Interval 0 (ASK MLC0): one particle every 2
updates (1 per FEP frame).

## 5. FecRadialg_line_particle (100) (COM BSS0 part 4)

Inputs: 0 = colour (Col), 1 = Lmatrix. Slots: 0 `0x576610`, 1 `0x5765a0` (returns 0), 2
`0x5765b0`, 3 `0x576730`, 4 `0x576710` (virtual dtor), 5 `0x576770` (returns 1). The object is
`sai::fec::particle::RadialGLine` (RTTI, vtable `0x7ecfb0`, base RadialG `0x7ecff0`).

### 5.1 Parameters (0x45 bytes; copied each call into a setup record by `0x564220`)

| off | type | meaning | COM BSS0 |
|---|---|---|---|
| 0x00 | vec4 | colour at the head (128 = 1) | (128, 96, 32, 128) |
| 0x10 | vec4 | colour at the tail | (128, 32, 32, 0) |
| 0x20 | f32 | speed (per update) | 0.2 |
| 0x24 | f32 | speed random +- | 0 |
| 0x28 | f32 | acceleration along the direction | 0 |
| 0x2c | f32 | acceleration random +- | 0 |
| 0x30 | f32 | gravity: subtracted from vel.y every update | 0 |
| 0x34 | f32 | emission rate | 1 |
| 0x38 | f32 | cone angle (degrees, full) | 80 |
| 0x3c | u16 | max particles | 100 |
| 0x3e | u16 | life (updates); 0 aborts the spawn | 10 |
| 0x40 | u16 | life random +- | 2 |
| 0x42 | u8 | n = points per line (position history) | 2 |
| 0x43 | u8 | emission interval (updates) | 12 |
| 0x44 | u8 | blend (as 4.1) | 1 |

### 5.2 State

Work `+0` -> RadialGLine (0x20): `+0x08` history storage (max x n x 0x20), `+0x10` pool (`+0`
particles, `+8` max, `+0xc` vertex bytes 16 x max x n), `+0x18` n, `+0x1c` f32 emission
accumulator. Particle (0x38): `+0x00` vec4 vel, `+0x10` vec4 acc, `+0x20` u16 life total, `+0x22`
u16 life left, `+0x24` f32 fade, `+0x28` u8 alive, `+0x30` -> its n history entries
{vec4 colour (0..1), vec4 pos}.

### 5.3 Update (`0x5765b0` -> `0x565870`, exact)

```
if ctx flag bit 0 or inst byte +0x0a or inst dt == 0: return
emit (4.5) with rate +0x34, interval +0x43; for each: first free slot at/after the running index
    h = wrap_pi(cone * pi/180 * 0.5)
    a1 = (2r-1) h; a2 = (2r-1) h                                    # radians
    d = (0,1,0,1) * (I x RotZ(a1 deg) x RotX(a2 deg))               # = (-sin a1, cos a1 cos a2, cos a1 sin a2, 1)
    acc = d * (accel + (2r-1) accelRand)                            # all four components
    vel = d * (speed + (2r-1) speedRand)
    p0  = (rand01 * vel.xyz, 1)
    life = u16(int((2r-1) * lifeRand) + life); total = life; fade = 1; alive = 1
    for k in 0..n-1: H[k].colour = colHead/128 + k * (colTail - colHead)/(128 n); H[k].pos = p0
for every alive particle:
    life -= 1; if life == 0: alive = 0; continue
    for k = n-1 down to 1: H[k].pos = H[k-1].pos                    # colours stay
    H[0].pos.xyz += vel.xyz
    vel += acc; vel.y -= gravity
    if life < (u16 +0x3e >> 1): fade = max(0, fade - 1 / (u16 +0x3e >> 1))    # vt[2] 0x563c20
```
RotZ `0x565b50` rows (c,s,0),(-s,c,0),(0,0,1), RotX `0x565a20` rows (1,0,0),(0,c,s),(0,-s,c);
both "in x R". So the particles leave the part origin into a cone of half-angle cone/2 about the
part's local +Y.

### 5.4 Draw (`0x576730` -> RadialGLine vt[1] `0x563c80`, exact)

```
Wm = 0x5b2190(ctx, part, part.local, lm b0, lm b1, lm b2, 1.0)
untextured preset, blend as 4.1, C = clamp(input0.vec4 * inst colour, 0, 255) / 255
for each alive particle (lines += 1):
    for k in 0..n-1:
        vertex.pos  = H[k].pos * Wm                       # transformed on the CPU here
        vertex.rgba = bytes of clamp01(H[k].colour * C, alpha * fade) * 255
shader [0xaaa940], format 2, world matrix = identity, 0x5e33e0(.., 0xff, 0, n, lines)
restore 0x55f850, submit
```
So each particle is a line from its current position back through its last n-1 positions (n = 2:
one segment of length |vel| per update), head colour at the newest point.

## 6. The Keyblade trail W_010_SO_LOC0_0

Element: body 5 of `w_010_so.fep`, 6 frames, looping, stop mode 3; one part (looping 0..2):
Pos 0, Rot 0, **Scl (100, 100, 70) %**, Colac (128, 96, 48, 150; a key at frame 2 starts its alpha
velocity/acceleration -66.3 / 14.4), Lmatrix bytes (1, 0, 0, 0, 0, 0) (matrix type 1, b1 = 0),
Wave_bindlocus_strategy, Bindlocus_polygon (params in 2.1 / 2.4; `tools/ddd_fep_tails_keyblade.py`).
Started by the Keyblade EAD on bone `Z_00`, case A (follow + bone rotation), offset 0, rot 0,
scale 1 (space-time 3.3), so the frame F = instance `+0x60` = Z_00's world matrix (rows normalised
x their lengths, i.e. W_Z00 itself, **INFERRED** no bone scale).

**The two points** (row vectors, `p_world = p * diag(1, 1, 0.7) * W_Z00`):

| edge | raw param | after y, z negation | x Lmatrix scale = Z_00 space (DDD) | KH1 units (x100) | Keyblade Root space |
|---|---|---|---|---|---|
| A (`+0x00`) | (0, 0, 1.05) | (0, 0, -1.05) | **(0, 0, -0.735)** | (0, 0, -73.5) | x = 0.0444 (4.4 from the grip) |
| B (`+0x10`) | (0, 0, -0.027) | (0, 0, 0.027) | **(0, 0, +0.0189)** | (0, 0, +1.89) | x = 0.7983 (79.8) |

**Z_00 in the Keyblade** (`w_so010.pmo`, 6 bones Root, wep, Z_00, chain1..3; PMO scale field
0.779): parent `wep` (at the Root origin, rows (1,0,0), (0,0,1), (0,-1,0)); Z_00's bind world =
translation **(0.7794, 0, 0)** in Root space, axes x = (0,0,1), y = (0,-1,0), **z = (1,0,0)**: Z_00
sits at the blade tip, 0.779 along Root's +x (the blade direction, which hangs along Sora's
`R_buki` +x, ddd-notes.md), with its own +z pointing out along the blade. The ribbon therefore
spans **the whole blade, from 0.044 (just above the grip) to 0.798 (0.019 past Z_00)**, i.e. from
5.7 % to 102.4 % of the grip -> Z_00 distance. A port feeds, every update, A = Z00_pos -
0.735 * z_dir and B = Z00_pos + 0.0189 * z_dir (DDD units; x100 for KH1), where z_dir is the
unit blade direction from grip to tip of KH1's Keyblade (scaled if the bone has scale).

Behaviour (all from 2): every update (PC: every 1/60 s) the two points are pushed into 6-sample
histories (no wave, no drift: rate, amplitude, drift 0), samples stay in world space (D =
identity), the first update lays all 6 at the start position (zero-area ribbon). The B-spline
(S 32) turns the 6 samples into 29 points per edge; the strip has 58 vertices. Texture F0000009
(128x128, resident), U across: A edge u = 0, B edge u = 64/128 = 0.5; V along: 1.0 at the newest
point down to 1 - 28/29 at the oldest, no scroll. Colour: head (128,128,128,128) x G = Colac x
element colour (so the Colac value itself), tail 0; head fade 7 %: k = int(0.07 x 28 + 1) = 2 ->
alpha x 0 at the newest point, x 0.5 at the next. Blend index 2 (additive, alpha weighted).
Depth offset on with (0, 0, -0.1) -> view-space z + 0.1 (space 3) at the Keyblade's frame origin:
depth-tested as if 0.1 DDD (10 KH1) units nearer the camera. `+0x4f` = 0 (viewport reset; no
visible effect on PC, **INFERRED**). Stop: the element gets a stop request at the EAD's t1 (motion
frames 25 / 34 for 322 / 323), mode 3 -> the part stops looping and dies at its life (6 frames);
on an earlier motion change an 8-tick handle fade (space-time 5.4), which reaches the ribbon
through the instance colour alpha.

## 7. Open points

- **Update rate** (2.3, 4.3, 5.3): the strategy samples, particle lives, velocities and spins are
  per pass-1 update, unscaled by dt; on PC that is per rendered frame at 60 fps (dt 0.5), twice per
  FEP frame (the data was authored for the 3DS, one update per FEP frame). **INFERRED** from the
  code and README 7.1; not checked with the game running. A port that matches PC DDD updates these
  at 60 Hz.
- Depth write and cull state during the Bindlocus and line-particle draws: these classes set only
  blend (and the line draws restore `+0x1c` / `+0x48` afterwards); the rest is whatever the state
  block holds (**GUESS**: depth test on, depth write as left by the previous effect draw, cull
  none as for the effect texrefs). The ribbon can be seen from both sides only without culling.
- Line primitive: 0xff -> 4 (or 2) in `0x671980`; that this is a line strip is **INFERRED**; line
  width (`0x601bd0` returns at once) is 1 pixel (**INFERRED**).
- Untextured preset `+0xf8` has combiner B = 0x201 (render-state.md 3): **INFERRED** RGB x1, alpha
  x255/128 for the line particles; their vertex bytes are already scaled to 255 = 1.0 (C / 255
  times colour / 128), so RGB 128 Colac gives half intensity unless the RGB scale is also 255/128.
  Check against render-state.md's VS formula before porting.
- Section 1.4's remark that a Locus trail in space 1 "does not stay in the world when the frame
  moves": with the same reasoning as 2.3 (W = M for a parentless part) the Locus frame delta is the
  identity too, so it **does** stay in the world. Not re-read in `0x5a5390`; worth a check.
- Cubic spline kind (`+0x53` = 0, eval `0x5a04b0` -> `0x59f1b0`) not decoded (unused in Flowmotion).
- `0x58c630` (random U/V start) **INFERRED** rand01 x argument; unused in Flowmotion.
- 1.8 (Shapetail orient) still TODO.

## 8. Tools

| tool | what |
|---|---|
| `tools/ddd_fep_tails_params.py [--raw] [--all [root]] [--ids ...]` | every parameter block of classes 68, 79, 80, 100, 104, 105, 106 in the seven Flowmotion FEPs, decoded with the field names of this doc, with the bytes past param_size; `--all`: histograms of the bytes past param_size over all FEPs |
| `tools/ddd_fep_tails_slot8.py` | section 3: references to the command table and the slot-8 functions, direct calls |
| `tools/ddd_fep_tails_slot8b.py` | section 3: every `call [reg+0x40]` in `.text` preceded by a `+0x1b` / `+0x44` read |
| `tools/ddd_fep_tails_keyblade.py` | section 6: Keyblade bind matrices, the trail's two points in Z_00 / Root space, B-spline count |
