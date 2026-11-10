# FEP generators, randomness and Colorblink

Topic spec for DDD's effect interpreter (`FunnyEffectsForPSP`): how parts spawn child parts
(FecGenerate 18, FecGenerateanchorvertex 150, FecGenerateanchor 151), the random number generator
every effect command uses, the Rnd* commands (22..25), and FecColorblink (62) with
FecColorblink_parameter (61). It builds on [README.md](README.md) (sections 5..8, 11) and corrects it
where noted (section 10).

**Sources.** DDD Steam exe, image base 0x140000000, **every address is an RVA**. Data: the seven
Flowmotion FEPs (README section 11) and, where marked "667", every FEP of the game. Disassembly
listings and survey outputs are kept in `a local work folder`.

**Labels.** Unmarked = read in the exe at the RVA given (and, where data is cited, checked on the
files). **INFERRED** = follows from read facts, not seen directly. **GUESS** = a guess.

Notation. `u()` = one draw of the unit random number (section 1.3). "FEP space" = the coordinates the
FEP's Pos commands use; "engine space" = after FecLmatrix's y/z negation (README 8.2). Work offsets
are byte offsets into the command's work area (instance `+0xa0 + 2*w`, README 6.2). "Input k" = the
k-th slot the command lists in its slot descriptor (README 6.2); its output is the first vec4 of its
work.

---

## 1. The random number generator

### 1.1 Algorithm: SFMT19937, one global instance

Every random draw of the effect commands is a call to `0x613020`, which is `jmp 0x6196a0`.

`0x6196a0` (read in full): `i = [0x14c8a2e0]`; if `i >= 0x270` (624) call `0x619730` (refill) and
set `i = 0`; return the u32 `state[i]` and store `i + 1`. The state pointer `[0x9ea8a0]` holds
VA `0x154c89920` = RVA `0x14c89920`: 624 u32 words ending exactly at the index word `0x14c8a2e0`.

`0x619730` (refill) is the **SFMT19937** recursion (SIMD-oriented Fast Mersenne Twister, Saito and
Matsumoto), unrolled two 128-bit words at a time:
`r = a ^ (a << 8 bits as 128-bit) ^ ((b >> 11) & MSK) ^ (c >> 8 bits as 128-bit) ^ (d << 18)` per
32-bit lane, with:

| SFMT parameter | value in the exe | where |
|---|---|---|
| N | 156 x 128 bits = 624 u32 | index limit `0x270` at `0x6196aa` |
| POS1 | 122 (b = a + 0x7a0 bytes) | `[rsi - 0x7a8]` / `[rsi - 8]` pairs at `0x619783`, `0x6197cc` |
| SL1 / SL2 | 18 / 1 byte | `shl eax, 0x12` `0x6197f4`; 64-bit `shl 8` / `shr 0x38` pairs `0x6197b1..0x6197bc` |
| SR1 / SR2 | 11 / 1 byte | `shr ebx, 0xb` `0x6197dd`; `shr rax, 8` `0x6197d9` |
| MSK1..4 | `0xdfffffef 0xddfecb7f 0xbffaffff 0xbffffff6` | after `>> 11` only the low 21 bits matter: the exe's masks `0x1fffef 0x1ecb7f 0x1affff 0x1ffff6` (`0x6197e0`, `0x619819`, `0x619838`, `0x619867`) are exactly those |
| parity | `{1, 0, 0, 0x13c9e684}` | `.data 0x9ea8b0`, used by the period check in `0x61a8d0` |

There is no tempering: SFMT outputs the state words as they are.

**Seeding.** `0x61a8d0` is SFMT's `init_gen_rand(seed)`: `state[0] = seed`,
`state[i] = 0x6c078965 * (state[i-1] ^ (state[i-1] >> 30)) + i` (`0x61a8e2..0x61a909`), then the
period certification with the parity table (`0x61a90b..0x61a9bc`) and index = 624 (`0x61a926`).
Its only caller is the thunk `0x6130a0`, called from:

- `0x60e7de` and `0x60ee1b`, with `seed = std::_Random_device()` (call `0x73919d` = import
  `MSVCP140.dll!?_Random_device@std@@YAIXZ` at `0x7824b0`): a **non-deterministic seed**;
- `0x43e3ee` and `0x43f4de`, with a seed drawn from the generator itself (`call 0x613020` just
  before).

`0x619640` / `0x619670` (seed through `0x619c00`, an init-by-array path) have no direct callers.
`0x613020` has 567 direct callers game-wide (`ddd_fep_fn.py -x 0x613020`). So the effects share one
generator with the rest of the game, seeded at random: **the exact sequence an effect sees cannot be
reproduced and does not matter.** A port needs only the per-draw formulas below and the number and
order of draws (given per command), with any good uniform generator.

### 1.2 Other entry points (not used by the generator commands)

| RVA | returns |
|---|---|
| `0x613030` | float in [a, b]: `a + (b - a) * float(u32) * 2^-32` (`[0x7f4f30]` = 2.32831e-10) |
| `0x613080` | float in [0, 1]: `float(u32) * 2^-32` |
| `0x6196e0` | u64: two consecutive state words, index += 2 |

### 1.3 The unit draw used by the FEP commands

Every FEP command read here converts a draw the same way (e.g. `0x58048a..0x580498`):

```
u = float32(r >> 1) * 2^-31          ; r = u32 from 0x613020, 2^-31 = [0x7ecf78] = 4.65661e-10
```

`r >> 1` is 0 .. 2^31-1; `cvtdq2ps` rounds it to 24 bits, so **u is in [0, 1], and 1.0 itself occurs**
(when `r >> 1 >= 2^31 - 64`, probability about 2^-25). The three range forms that recur:

| form | code (single precision, in this order) | range |
|---|---|---|
| sym(R) | `t = u + u; t = t * R; t = t - R` | [-R, R] |
| pos(R) | `h = R * 0.5; t = u + u; t = t * h; t = t - h; t = h + t` | [0, R] |
| neg(R) | `h = R * 0.5; t = u + u; t = t * h; t = t - h; t = t - h` | [-R, 0] |

(0.5 = `[0x867d7c]`.) A port that wants bit-exact arithmetic follows the order shown.

### 1.4 Which Flowmotion command classes draw

From `tools/ddd_fep_generators_rngmap.py --fm` (direct calls only, up to 3 levels below a vtable
slot). The classes of this topic are decoded below; the others are listed for the other topics.

| class | slot -> function | draws per call |
|---|---|---|
| 151 Generateanchor | 1, 2 -> `0x580350` | 3 per child (section 3) |
| 18 Generate | 2 `0x581a20` | 3 or 4 per child (section 4) |
| 150 Generateanchorvertex | 1, 2 -> `0x57fb70` | 0 or 1 per child (section 5) |
| 22 Rndpos, 23 Rndrot | 1 | 3 per enabled component group (section 6) |
| 24 Rndscl | 1 | 1 or 3 per enabled group |
| 25 Rndcol | 1 | 4 per enabled group |
| 61 Colorblink_parameter | 2 -> `0x584cd0` | 4 per fade start, only if its random flag is set (section 7) |
| 155 Mdlts_plus | 1 `0x588c80` | up to 6 (not decoded here) |
| 152 Anm_plus | 0 `0x586dc0` | up to 4 (random start, flip; README 8.6) |
| 153 Megaanm_plus | 2 `0x58cdd0` and callees | many (not decoded here) |
| 105 Bindlocus_polygon | 0 -> `0x58c630` | 1 |
| 106 Wave_bindlocus_strategy | 2 -> `0x5a1060` | 3 |
| 68 Locus_tail_strategy | 0, 5 -> `0x5a6d60` | 1 |
| 80 Shapetail | 0 `0x5a24c0`, 1 `0x5a2200` | 1, 2 |
| 104 Ellipse_line_particle | 2 `0x55fa20` | 7 |
| 100 Radialg_line_particle | 2 -> `0x565110` | 6 |

---

## 2. Spawning a child: the shared code

The three generator classes spawn children with the same steps. Generateanchor's copy is
`0x580350`; Generate's is inside `0x581a20` (`0x581c28..0x582085`); Generateanchorvertex's is in
`0x57fb70` (`0x57fd06..0x58021e`).

### 2.1 Buffers and spawn info

On the stack, 16-aligned: `pos`, `vel`, `acc` (three vec4), `P` (vec4 offset, `w = 1.0`), `M`
(4x4 matrix). The spawn info passed to the child is `{u32 flags = 7, &pos, &vel, &acc}`
(`0x580405..0x580423`, `0x581c43..0x581c55`, `0x57fd31..0x57fd5e`): bits 0, 1, 2 = position,
velocity, acceleration (README 7.3).

### 2.2 Per child

1. **Offset P.** Generate and Generateanchor: `P = (sym(c), sym(c), sym(c))`, c = param `+0x04`,
   three draws in x, y, z order (`0x580485..0x580507`; `0x581cff..0x581d7f`). Generateanchorvertex:
   P = a model vertex (section 5). The draws happen **even when c = 0**.
2. **Matrix M.** Generate and Generateanchor: identity (`0x63d6c0`) with translation row =
   input 0's output with **y and z negated**: `(x, -y, -z, 1)` (`0x58050d..0x58054c`,
   `0x581d84..0x581dc8`). Input 0 is the generator's Pos/Posac (data: 436 + 33 blocks). M is rebuilt
   for every child (the input is re-read each time). Generateanchorvertex: M = the generator part's
   **local matrix** `+0x78` (`0x57fc93`), not an input.
3. **Frame F.** The generator part's frame owner's own frame `[owner + 0x88]` if the part has a
   frame owner (`+0x28`), else the element's effect matrix (element instance `+0x60`)
   (`0x580593..0x5805b0`).
4. **Child matrix type** t = the child part's byte `+0x0b` (0 when the part entry is missing)
   (`0x58055b..0x580593`).
5. **Placement** `0x5b23b0(ctx, t, &P, &M, F, &pos, &vel, 1)` (read in full):

   | child type t | pos (engine space) | direction d (before normalising) |
   |---|---|---|
   | 0, 2..8 | P | P |
   | 1 | P x M (row vector, `0x58c550` -> `0x63ad40`) | P x M - M.row3 = P x M3x3 |
   | 9 | P x (M x F) (`0x58c5b0` -> `0x63ac90`) | P x (M x F)3x3 |
   | < 0 or > 9 | P | P (never in the files) |

   The last argument 1 selects `d = normalize(d)` (`0x5b2591 -> 0x5b262f -> 0x58c6c0`): x, y, z
   divided by the length, `w = 1`; **a zero vector stays (0, 0, 0)** (`0x58c6fd..0x58c709`).
   (Argument 2, used by other callers, also scales d by the row lengths of M: `0x5b25a0..0x5b2626`.)
6. **Velocity and acceleration** (`0x5805d7..0x580608`):
   `acc.xyz = d * param_acc` first, then `vel.xyz = d * param_speed` (`0x63c240`; the w words are
   kept). Generate / Generateanchor: speed `+0x08`, acc `+0x0c`; Generateanchorvertex: speed
   `+0x04`, acc `+0x08`.
7. **Back to FEP space:** y and z of pos, vel and acc are multiplied by -1 (`[0x867f78]` = -1,
   `0x580612..0x58066f`). The child's own FecLmatrix negates them again (README 8.2).
8. **Spawn** `0x5b7980(ctx, child ref, start time 0.0, &spawn info)` (`0x580675..0x580679`):
   `0x5b73d0(ctx, element, parent = the generator's part instance, ref, 0.0, info)` and then restores
   `ctx+0x80` / `ctx+0x88` (`0x5b79d0..0x5b79dc`). The ref is passed whole (bits 0..11 part index,
   bit 13 states, README 5). A child with a negative ref returns null at once (`0x5b79a0`).
   **If the spawn returns null, the rest of the burst is dropped** (`0x580681 -> 0x58081a`; same in
   `0x581efe`, `0x58006f`).

Spawning happens during the pass-1 update (phase 8..9), so the child's key lists up to int(0) = its
frame-0 keys run at once inside the spawn (`0x5b793c..0x5b7961`, README 7.2), and the child, linked
at the tail of the pass list, gets its first update slots in the same pass (commands.md 5.3,
**INFERRED**).

### 2.3 Direction: what it means

- The speed direction is **from the emitter centre through the random point P of the cube**,
  normalised. It is not uniform on a sphere: directions bunch toward the cube's diagonals (the
  density is that of a uniform cube projected radially). The magnitude is exactly `speed`; a
  negative speed points inward (one Generate block in Flowmotion has speed -0.12).
- **With cube size 0 there is no motion at all**, whatever speed and acceleration say: P = 0 gives
  d = 0. This is common: 24 of 436 Generateanchor blocks in the Flowmotion files (10102 of 38749
  game-wide) have cube 0 and speed or acceleration 1.0; 27 of 33 Generate blocks have cube 0.
- Space: types 0 and 2..8 keep P and d in the child's own local space; the parent step of FecLmatrix
  (type 0: translation by the parent's local matrix; 3: times the parent's local matrix; 4: parent
  position) or the frame captured at spawn (2, 6, 7, 8) carries them later (README 8.2, commands.md
  7.2, 7.3, **INFERRED** for the composition). Type 1 adds the emitter's Pos output and nothing else
  (no rotation, no parent): its frame is the live effect matrix or the owner's frame. Type 9 bakes the
  full frame into a world-space position and direction (its own frame is identity, `0x5b77ee`).
- Child types met in the Flowmotion files (per block): Generateanchor 3: 292, 1: 49, 0: 27, 4: 12,
  6: 4; Generate 3: 18, 1: 7, 6: 6; Generateanchorvertex 0: 10.

### 2.4 How the child takes the spawn info: FecPosac init (`0x582e10`)

Every generated child in the Flowmotion files positions itself with FecPosac (228 of 228 child
blocks; `ddd_fep_generators_survey.py`). Its slot 0 copies the parameter value, velocity and
acceleration (vec3, w = 0) to work `+0x10`, `+0x20`, `+0x30`, then, for each set bit, **adds**
`pos` to `+0x10` (bit 0, `0x582e6f..0x582ebb`), `vel` to `+0x20` (bit 1, `0x582ec6..0x582f12`),
`acc` to `+0x30` (bit 2, `0x582f1d..0x582f69`), setting that vector's w to 1.0; output `+0x00` =
`+0x10` (`0x582f6e`). Pos (`0x583570`) and Posmv (`0x583290`) replace instead (README 7.3; not used
by generated children here). The ac key change (README 8.1) carries the spawn offset over into
later keys.

### 2.5 Part groups (ref bit 12): never used

`0x5b0e80(ctx, body, ref)` returns -1 unless ref >= 0 and bit 12 is set; then the count of part
group `ref & 0xfff` (`0x5b0e85..0x5b0eb3`). With a group, each burst child draws P once and spawns
**every member** with that same P: member j's ref is `0x5b0ec0(body, ref & 0xff, j)` and the spawn is
`0x5b0e20` (same arguments; **its null return is not checked**). Note the group index is taken as
`ref & 0xfff` for the count but `ref & 0xff` for the members (`0x5806ae`). No generator block in 667
files sets bit 12 (README 5), so a port can ignore groups.

---

## 3. FecGenerateanchor (151): one burst per key

### 3.1 Parameters (0x15 bytes; README 3 param_size)

| off | type | meaning (reader) | Flowmotion values (436 blocks) |
|---|---|---|---|
| 0x00 | f32 | children per burst, truncated (`cvttss2si` `0x580435`, re-read each loop `0x580808`) | 1 (372), 0 (64) |
| 0x04 | f32 | cube half-size c (`0x580485`) | 0 (428), 0.3 (8) |
| 0x08 | f32 | speed (`0x5805f6`) | 0 (412), 1 (24) |
| 0x0c | f32 | acceleration (`0x5805d7`) | 0 (412), 1 (24) |
| 0x10 | s16 | child part ref; < 0 = off (`0x58037d`) | -1 in 52 blocks |
| 0x12 | s16 | < 0 = off (`0x580383`) | 0 (436; 38749/38749 game-wide) |
| 0x14 | u8 | no reader | 0 (game-wide 2 in 8 blocks) |

### 3.2 Slots (work: u8 pending)

| slot | RVA | action |
|---|---|---|
| 0 init | `0x580340` | pending = 0; return 1 |
| 1 key | `0x5802d0` | if pending and old params != null: burst with the **old** params; then pending = 1 |
| 2 update | `0x580310` | if pending: burst with the current params, pending = 0 |
| 5 loop | `0x580890` | pending = 1; return 1 |

Effect: **each key list that names the slot causes exactly one burst with its own block**, in the
first pass-1 update at or after that key (keys run before phase 2 in `0x5b4700`). If two key lists
name it in one update, the first one's burst fires from slot 1 when the second arrives. The init
block never fires (slot 0 clears pending, the frame-0 key sets it). Each loop wrap fires one burst
(slot 5), with the class-matched block of section 3.5.

### 3.3 The burst (`0x580350`)

```
if s16 p[0x10] < 0 or s16 p[0x12] < 0: return          ; no draws
in0 = input 0's work offset (0 if the slot has no inputs)   ; 0x5803b3..0x5803cf
groups = 0x5b0e80(ref)                                   ; -1 in every file
M = identity
for i in 0 .. int(p[0x00]) - 1:                          ; condition re-read each pass
    P = (sym(c), sym(c), sym(c)), w = 1                  ; 3 draws: x, y, z
    M.row3 = (in0.x, -in0.y, -in0.z, 1)
    place, scale, negate, spawn (section 2.2)
    if spawn failed: stop
```

Draws per burst: 3 x count, in the order x, y, z per child.

### 3.4 AJC SHO0 part 16 and its child -1 slots

Part 16 of P_FA_AJC_SHO0_0 (the dive's ring emitter; mtype 1, life 29, spawned at frame 2) has nine
Generateanchor slots 4..12, each with input 0 = slot 0 (FecPos). Their init and frame-0 key blocks
are equal (`ddd_fep_generators_survey.py`):

| slot | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|
| child | 12 | -1 | 13 | -1 | -1 | -1 | 14 | 15 | -1 |

All have count 1, cube 0, speed 1, acceleration 1. So **five of the nine slots (not one) are off**:
a negative child returns at `0x58037d` before any work or random draw, so they do nothing at all.
Why they exist cannot be read from the exe; **GUESS**: authoring leftovers (emitters whose child
part was deleted in the editor, which then writes -1). The four live slots each spawn one child of
matrix type 3 at P = 0 with no motion (cube 0), i.e. exactly at part 16's local matrix: rings 12,
13, 14, 15 start at frame 2 on part 16's position. Game-wide 4220 of 38749 Generateanchor blocks
have child -1.

### 3.5 Loop wraps: the block a slot fires with

At a wrap the loop rewind (`0x5b6680`) gives each loop-list slot the block found by
`0x5b80c0(part, class id, loop start)` (read in full): over the key lists in order while
`list.frame <= loop start`, take the **first entry whose slot has the same class id** (`0x5b8198`);
the last such list wins; with no match, the first init entry of that class (`0x5b81df..0x5b8262`).
`+0x68[slot]` = that block (`0x5b67da`), slot 5 runs, and slot 2 then bursts with it. So in a part
with several Generateanchor slots, **all of them fire the same block** after a wrap: the first
Generateanchor entry of the latest key list at or before the loop start.

`ddd_fep_generators_survey.py --loop` lists every case in the 19 elements. Examples where the result
differs from the slot's own block:

| part | own blocks before the loop start | block used after every wrap | visible result |
|---|---|---|---|
| ASK MLC0 p6 (loop 3..4) | slots 5, 6, 8, 9 count 1 (children 1, 2, 4, 5) | slot 7's frame-3 block: count 0 | no bursts after the first wrap |
| ASK MLC0 p18 (loop 3..4) | slots 5, 6, 7, 8, 10 count 1 | slot 4's frame-2 block: count 0 | no bursts after the first wrap |
| COM LOC1 p0 (loop 1..2) | slot 9: count 1, child -1 | slot 8's frame-1 block: count 0, child 5 | none (same as own) |
| COM LOC1 p2, p3 (loop 1..2) | slot 5: count 1, child 1 | slot 4's frame-1 block: count 0 | slot 5 stops |

In all listed cases the matched block has count 0, so after the first wrap these emitters are
silent: emulating the class match matters only in that it must **not** re-fire the slots' own
count-1 blocks.

Side note (loop rewind, outside this topic): `0x5b80c0`'s "entry" output is the last entry it
**examined**, not necessarily the matched one (`r9` at `0x5b8171` keeps moving after a match), and
the init fallback returns the init entry, so `t_new` = loop start - 0, not 0 as README 7.2 says.
Neither matters for the generators (they ignore t_new and the entry).

---

## 4. FecGenerate (18): continuous emission

### 4.1 Parameters (0x18 bytes)

| off | type | meaning (reader) | Flowmotion (33 blocks) |
|---|---|---|---|
| 0x00 | f32 | rate: children per frame (`0x581a6c`, `0x581bbd`, `0x581c1c`) | 1 (20), 0 (9), 2, 3 |
| 0x04 | f32 | cube half-size (`0x581cff`) | 0 (27), 0.4 (5), 0.5 |
| 0x08 | f32 | speed (`0x581e73`) | 0 (27), 0.1 (5), -0.12 |
| 0x0c | f32 | acceleration (`0x581e54`) | 0 (27), -0.002 (5), 0.002 |
| 0x10 | s16 | child ref; < 0 = off (`0x581a43`); bit 12 = part group (`0x5b0e80`, never set) | -1 in 2 |
| 0x12 | u16 | interval: frames of silence after each emission (`0x581b52`, `0x581bcf`, `0x581c11`) | 0..10 |
| 0x14 | s16 | < 0 = off (`0x581a60`) | 0 (8625/8625 game-wide) |
| 0x16 | u8 | no reader | 0 (2 in 60 blocks game-wide) |
| 0x17 | u8 | chance in percent per child; >= 100 = always (`0x581cd1`) | 100 (33); game-wide 8228 x 100, also 0..90 |

### 4.2 Slots (work: f32 acc)

| slot | RVA | action |
|---|---|---|
| 0 init | `0x582110` | acc = 0; return 1 |
| 1 key | `0x5819f0` | if `ctx+0xa0 > 0`: acc += old params' rate; else acc = 0 |
| 2 update | `0x581a20` | below |
| 5 loop | `0x582120` | return 1 (acc is kept across wraps) |

`ctx+0xa0` counts the key lists already run in the current call of the key runner: it is zeroed at
`0x5b518d` and incremented after each list at `0x5b52d1`. So the **first key list in an update resets
acc**; each further key list caught up in the same update adds one old rate.

### 4.3 Update (`0x581a20`, read in full)

Arguments: params p, local time t (xmm3), work acc. dt = the element's `+0x13c` (the instance dt,
README 7.1). Part time `+0x40`, lifetime `+0x44`, part flags `+0x04` of the generator's own part
instance.

```
if s16 p.child < 0 or s16 p[0x14] < 0: return
if p.rate == 0 and acc == 0: return
if dt == 0: return
if dt < 0:                                   ; 0x581ae9 (no file reaches it; INFERRED never)
    total = p.rate; goto EMIT
steps = dt
if not (part.flags & 2):                     ; not a loopable part
    if part.time + dt >= part.lifetime:      ; this step reaches the end of life
        left = lifetime - time
        if dt > 1: left = max(1, left)
        steps = left
        if left > 1:                         ; only possible when dt > 1
            I = p.interval if p.interval > 0 else 1
            k = 0; do { left -= I; k += 1 } while left > 1
            steps = k
if t < 1 and acc == 0 and p.rate == 1 and dt < 1 and dt > 1 - (dt + t):
    acc = 1 - dt                             ; 0x581b82..0x581bb9: first-frame catch-up
total = p.rate * steps + acc
threshold = 1 if p.interval > 0 else p.rate
if total < threshold: acc = total; return
if dt < 1: total = p.rate                    ; 0x581be7
EMIT:
n = trunc(total)
acc = total - n - p.interval * p.rate        ; 0x581bf7..0x581c24
for i in 0 .. n-1:
    if p.chance < 100:                       ; 1 draw
        if not (p.chance > trunc(u() * 100)): continue     ; 100 = [0x867f14]
    P = (sym(c), sym(c), sym(c))             ; 3 draws
    place, scale, negate, spawn (section 2.2); if spawn failed: stop
```

What this gives, at dt = 1 (one call per FEP frame): with interval 0, `rate` children per frame
(fractions accumulate); with interval I > 0, `int(rate)` children, then I silent frames: **one
emission every I + 1 frames** (acc drops to `frac - I*rate` and climbs back by `rate` per frame
until it reaches 1). At dt = 0.5 (**GUESS**: 60 fps, README 7.1) the `dt < 1` rules keep the same
period in frames (each emission spawns exactly `int(rate)`), and the first-frame rule makes the first
emission come at local time 0.5 at the latest. The end-of-life rule only changes anything when one
step jumps past several frames (dt > 1).

Chance: a child spawns when `chance > trunc(u * 100)`, so with probability chance/100 (exactly; a
`u = 1.0` draw gives 100 and fails even for chance 100, but chance >= 100 skips the test). A failed
chance skips that child's three cube draws.

### 4.4 The six Generate commands of the 19 elements

| element : part (generator) | child (type, life) | blocks | behaviour |
|---|---|---|---|
| AKD DRL0 : p4 (mtype 255, life 1, loops 0..1) | p3 (1, 20) | rate 1, interval 7 | one child every 8 frames while the effect loops |
| AKD DRL0 : p9 (same) | p8 (1, 12) | rate 1, interval 5 | every 6 frames |
| COM BSS0 : p5 (mtype 1, loops 0..1) | p4 (6, 12) | key0 rate 1 int 1; key1 (frame 1) rate 0 | every 2 frames; key1 is never reached while looping (the wrap at time 1 comes first), so it turns the emitter off only once the effect stops looping (**INFERRED**) |
| COM BSS0 : p9, p10 (mtype 1, loops 0..1) | p8 (3, 15) | key0 rate 1 int 10; key1 rate 0 int 5 | every 11 frames, same key1 remark |
| COM LOC1 : p0 slot 5 | -1 | rate 1 | off (child -1) |

All six have cube 0: the children appear at the emitter with no velocity. The wrap gives each the
class-matched block, which here is always the slot's own frame-0 block (only one Generate per part).

---

## 5. FecGenerateanchorvertex (150): children on model vertices

### 5.1 Parameters (0x17 bytes)

| off | type | meaning (reader) | Flowmotion (10 blocks) |
|---|---|---|---|
| 0x00 | f32 | children per burst, truncated (`0x57fd6e`, `0x58021e`) | 2 (8), 5 (2) |
| 0x04 | f32 | speed (`0x57ffdb`) | 0 |
| 0x08 | f32 | acceleration (`0x57ffbc`) | 0 |
| 0x0c | s16 | child ref; < 0 = off (`0x57fb8e`) | 0, 1, 2, 18 |
| 0x0e | s16 | **section-3 (MDL) index** in the element's own FEP; < 0 = off (`0x57fbb1..0x57fbd6`) | 0, 6, 9 |
| 0x10 | s16 | **section-5 (VTL) index**; < 0 = off (`0x57fc8a..0x57fcd5`) | 0, 1 |
| 0x12 | s16 | list index inside the VTL (`0x57fc8e`, no bounds check) | 0, 1 |
| 0x14 | s16 | < 0 = off (`0x57fba5`) | 0 (1573/1573 game-wide) |
| 0x16 | u8 | bit 0: 1 = take the list's vertices in order, 0 = at random (`0x57fdd0`) | 1 (10); game-wide 1495 x 1, 78 x 0 |

### 5.2 Slots (work: s16 cursor at +0, u8 pending at +2)

Same pattern as Generateanchor: slot 0 `0x57fb60` cursor = 0, pending = 0, return 1; slot 1
`0x57faf0` bursts with the old params if pending, then pending = 1; slot 2 `0x57fb30` bursts if
pending, then pending = 0; slot 5 `0x5802c0` pending = 1. The cursor is reset only at init.

### 5.3 The burst (`0x57fb70`, read in full)

```
if s16 p[0x0c] < 0 or s16 p[0x14] < 0 or s16 p[0x0e] < 0: return
file = element instance +0x198; rec = MDL record p[0x0e] (section 3 table, file +0x34)
if file +0x34 == 0 or rec.slot (+0x10) == -1: return
scale0 = rec.f32 (+0x00); centre = rec.u32 (+0x14) & 1          ; 0x57fc38, 0x57fc44
model = MDL manager slot rec.slot (array mgr +0x30, 32-byte slots, 0x1c0 of them; 0x5bfa40,
        0x5bfa50); data = [model + 8] = the MDL header (0x5bf5d8)
L = the generator part's local matrix (+0x78)
if s16 p[0x10] < 0: return
list, n = 0x5c0a60(Fermdl manager ctx+0xc0, VTL record slot, p[0x12])
          ; VTL data = [[mgr + 0x28 + 32*slot] + 8]; table = data + [data+0x18];
          ; n = table[i].u32 count, list = data + table[i].offset (null if 0)
if list == null: return
mesh = data + [data + 0x1c]          ; FIRST mesh only
pos  = mesh + [mesh + 4]             ; s16 x, y, z per vertex (offset relocated by CMdl::Load 0x5bf80e)
k = scale0 * f32 mesh[0x24]
for i in 0 .. int(p[0x00]) - 1:
    if p[0x16] & 1: j = cursor; cursor += 1; if cursor >= n: cursor = 0       ; in order, wraps
    else:           j = trunc(u() * n)                                        ; 1 draw
    v = s16 list[j]
    if centre: P = ((pos[3v+a] * 2^-15 - f32 mesh[0x38 + 4a]) * k for a = x, y, z)
    else:      P = (pos[3v+a] * 2^-15 * k for a = x, y, z)                    ; 2^-15 = [0x7ecfa0]
    M = L; place, scale (speed +0x04, acc +0x08), negate, spawn (section 2.2); if failed: stop
```

So the vertex list is a **VTL list of a model chosen by the generator's own parameters** (section 3
index `+0x0e` of the element's FEP, which may name a resident model), not the parent part's draw
model. Data check over all 667 FEPs (1472 blocks that get past the early returns): the model always
has exactly one mesh, the list is never empty and every vertex number is below mesh 0's position
count; all 69 blocks with a negative list index also have a negative MDL index (early return). `count
> n` occurs in 12 blocks (the in-order cursor then wraps). MDL record `+0x14` is 0 in all 7608 records
(centring never used) and `+0x00` is 1.0 (0.95 in 3). Draws: one per child only when bit 0 is clear
(never in Flowmotion).

### 5.4 The model data it reads

`ddd_fep_generators_survey.py --mesh` over every MDL mesh of every FEP and resident.frr (2787
meshes): **mesh `+0x00` = number of positions, `+0x04` = offset (from the MDL start in the file) of
s16 x, y, z triples; `+0x38..+0x40` = their mean / 32768 in all 2787 (largest error 1.4e-5); and the
largest |s16| is exactly 16384 in all 2787**. So the positions are normalised to +-0.5 of 2^15 and
`+0x24` (1.0, 2.0, 1.2, 10, 20, 42.7, ...) is the size: a vertex in model units is
`s16 / 32768 * mesh[0x24]` (times the record's f32). This position table is what the generator reads;
whether the drawn vertices (README 9.4) use the same scale is **INFERRED** (they span the same
-16384..16384), and if so README 9.4's "16384 = 1.0" is off by `mesh[0x24] / 2`.

### 5.5 The five uses in the 19 elements

| element : part (mtype, life) | slots -> child (type) | model, VTL list | vertices (P, model units) |
|---|---|---|---|
| AJC CAL0 : p2 (1, 21) | 4 -> p0, 5 -> p1 (0) | MDL 0 = resident F000012E (6 points), VTL F0000186 list 1 = [0, 4] | (-0.5, 0, 0), (0.5, 0, 0) |
| AKD CAL0 : p3 (0, 19) | 4 -> p1, 5 -> p2 (0) | MDL 6 = F000012E, same list | same |
| AJC SHO0 : p19 (1, 20) | 4 -> p18 (0) | MDL 9 = local 9803AD09, VTL 9803AD81 list 0 = [1, 2, 3, 4, 0] | regular pentagon of radius 0.5 in the y = 0 plane: (-0.476, 0, -0.155), (-0.294, 0, 0.405), (0.294, 0, 0.405), (0.476, 0, -0.155), (0, 0, -0.5) |

(Those P are engine space; the spawn hands the child `(x, -y, -z)` and the child's FecLmatrix flips
them back, so a type-0 child sits on the vertex as transformed by the generator's local matrix.)
With count 2 on a 2-vertex list and in-order selection, each CAL0 burst puts one child on each end
of the line; the SHO0 burst puts the trail part 18 on all five pentagon corners (count 5, in order).
Speed and acceleration are 0: the children do not move by themselves.

---

## 6. FecRndpos (22), FecRndrot (23), FecRndscl (24), FecRndcol (25)

Slot 0 of each returns 1 (`0x5a98a0`, `0x5aa220`, `0x5aabf0`, `0x5a8f10`). All the work is in slot 1
(key), which is **skipped once the part has looped** (`test byte [part + 2], 2`: `0x5a9527`,
`0x5a9eb7`, `0x5aa817`, `0x5a8a97`). The target is input 0 (a Pos*, Rot*, Scl* or Col* command); it
modifies the target's work directly. Because the target is an earlier slot, its own key change has
already run.

### 6.1 Offset tables (bytes at `0x9e82fc..0x9e8336`, dumped)

| class | enable byte offsets | mode byte offsets | range offsets | components |
|---|---|---|---|---|
| Rndcol | `0x9e82fc`: 0x30, 0x32, 0x34 | `0x9e8300`: 0x31, 0x33, 0x35 | `0x9e8304`: 0x00, 0x10, 0x20 | 4 (r, g, b, a) |
| Rndpos | `0x9e830c`: 0x0c, 0x0e, 0x1c | `0x9e8310`: 0x0d, 0x0f, 0x1d | `0x9e8314`: 0x00, 0x10, 0x20 | 3 |
| Rndrot | `0x9e831c`: 0x0c, 0x0e, 0x1c | `0x9e8320`: 0x0d, 0x0f, 0x1d | `0x9e8324`: 0x00, 0x10, 0x20 | 3 |
| Rndscl | `0x9e832c`: 0x0d, 0x0f, 0x1d | `0x9e8330`: 0x0e, 0x1c, 0x1e | `0x9e8334`: 0x00, 0x10, 0x20 | 3 (+ byte 0x0c) |

The three entries are k = 0 value, 1 velocity, 2 acceleration. (The tables are interleaved with
`0d 0e 0f` at `0x9e8308`, `0x9e8318`, `0x9e8328`, **GUESS** the per-axis mode bytes of Rndpos1 and
its siblings; `0x9e82f0` holds 1/255 twice.) This confirms commands.md 6.6.

### 6.2 Exact formulas

For k = 0, 1, 2 in that order, if `p[enable_k] != 0`:

- **Rndpos / Rndrot** (`0x5a95c0..0x5a9828`, `0x5a9f50..0x5aa1b1`): three draws, **always**, in the
  order x, y, z, then by `m = p[mode_k]`: m = 1 -> pos(R), m = 2 -> neg(R), anything else
  (0 or >= 3) -> sym(R), per axis with R = the range vec3.
- **Rndscl** (`0x5aa8b0..0x5aab7d`): if byte `p[0x0c] == 0`: **one** draw, `d.x` by the mode with
  range.x, and `d.y = d.z = d.x` (uniform scaling); else three draws x, y, z, each with its own
  range component. Modes as above.
- **Rndcol** (`0x5a8b50..0x5a8e86`): four draws r, g, b, a; modes as above; vec4 ranges. No clamp.

Then the target's work (offsets of the value / mv / ac layout, README 8.1):

| k | Rndpos, Rndrot | Rndscl | Rndcol |
|---|---|---|---|
| 0 value | base `+0x10` += d; output `+0x00`.xyz = new base | base += d; output.xyz = new base * 0.01 (`[0x867d08]`) | base += d (4 lanes); output = new base |
| 1 velocity | `+0x20` += d | same | same (4 lanes) |
| 2 acceleration | `+0x30` += d | same | same (4 lanes) |

Draw count per key: Rndpos/Rndrot 3 per enabled k; Rndscl 1 or 3 per enabled k; Rndcol 4 per
enabled k. In the 19 elements only k = 0 with mode 0 is used, except Rndrot velocity (COM WAL0, range
(0, 0, 5)) and Rndscl mode 1 (2 blocks); Rndpos is present but enables nothing in the 19 elements
(`ddd_fep_generators_survey.py --rnd`).

---

## 7. FecColorblink_parameter (61) and FecColorblink (62)

In every file each Colorblink has input 0 = a Colac or Colmv and input 1 = a Colorblink_parameter
(Flowmotion: 8 of 8 Colac; game-wide 1164 Colac, 420 Colmv), and nothing reads Colorblink
(1584/1584). The parameter command has no inputs.

### 7.1 Colorblink_parameter (61): a blink factor

Parameters (0x4a bytes):

| off | type | meaning |
|---|---|---|
| 0x00 | vec4 f32 | colour A (factor: 1.0 = unchanged) |
| 0x10 | vec4 f32 | random range for A |
| 0x20 | vec4 f32 | colour B |
| 0x30 | vec4 f32 | random range for B |
| 0x40 | u16 | hold A, frames |
| 0x42 | u16 | hold B, frames |
| 0x44 | u16 | fade, frames (both directions) |
| 0x46 / 0x47 | u8 | A: random on / kind |
| 0x48 / 0x49 | u8 | B: random on / kind |

Work: `+0x00` vec4 output factor, `+0x10` vec4 step, `+0x20` u32 counter, `+0x24..+0x30` u32
phase ends `e0 = holdA`, `e1 = e0 + fade`, `e2 = e1 + holdB`, `period = e2 + fade`, `+0x34` u32
current phase.

| slot | RVA | action |
|---|---|---|
| 0 init | `0x5852c0` | counter = 0, phase ends from the params, output = (1, 1, 1, 1), step = 0, phase = 0; return 1 |
| 1 key | `0x584ff0` | the same reset with the new params |
| 2 update | `0x585050` | below |
| 4 release | `0x585330` | nothing |
| 5 loop | `0x585340` | return 1 (the blink runs on across wraps) |

Update (`0x585050..0x5851d6`, read in full):

```
if ctx.flags & 1 or element[+0x0a] != 0 or element.dt == 0: return
ph = 0 if counter < e0 else 1 if counter < e1 else 2 if counter < e2 else 3
if ph != stored phase:
    if ph == 1:                 TARGET(B); (output moves below)
    if ph == 3:                 TARGET(A)
    if ph == 2 and stored == 0: TARGET(A); output += step; step = 0      ; fade 0 jump
    if ph == 0 and stored == 2: TARGET(B); output += step; step = 0      ; fade 0 jump
    otherwise (entering a hold after its fade): step = 0
    stored phase = ph
output += step                  ; all four lanes
counter += 1; if counter >= period: counter = 0
```

`TARGET(X)` (`0x5851e0` for B, `0x585250` for A, both -> `0x584cd0`):

```
if X.random_on == 0: R = (0, 0, 0, 0)
elif X.kind == 2:    R = -sym(X.range) per lane          ; 4 draws r, g, b, a; 0x584d2f
else:                R =  sym(X.range) per lane          ; kinds 0, 1 and >= 3; 0x584dfb
T = clamp(X.colour + R, 0, 1) per lane                    ; 1.0 = [0x867db0]
step = (T - output) / fade if fade > 0 else T - output    ; 0x584f78..0x584fe3, 0x63b100
```

So the kinds do not give one-sided ranges as in the Rnd* commands: all are symmetric (kind 2 only
mirrors the draw). Targets are clamped to [0, 1]: an A alpha of 1.3 or 2.0 (used outside Flowmotion)
acts as 1.0.

In time: hold A for `holdA` updates (the first time at (1, 1, 1, 1), the init value, not A), fade
to B over `fade` updates, hold B, fade back to A, repeat. **With fade 0 the roles swap** (a quirk of
the jump cases): the output jumps to A when the counter reaches holdA and to B when it wraps, so A
shows for holdB updates and B for holdA updates.

The counter advances **once per update call, whatever dt is** (only dt = 0 stops it). **INFERRED**:
if the update runs twice per FEP frame (dt = 0.5), the blink runs twice as fast in game time.

### 7.2 Colorblink (62): applying the factor

| slot | RVA | action |
|---|---|---|
| 0 init | `0x584c70` | own work = copy of input 0's output; return 1 |
| 1 key | `0x584af0` | own work = copy of input 0's output |
| 2 update | `0x584b40` | input 0's output = clamp(input 0's output * input 1's output, 0, 255) per lane (`0x63b0c0` multiplies lane by lane; 255 = `[0x78e78c]`) |
| 4 release | `0x584cb0` | nothing |
| 5 loop | `0x584cc0` | return 1 |

The colour is changed **in place**, in the Colac's output, after the Colac's own update recomputed it
(the Colac is an earlier slot), so the factor does not compound. Colorblink's own work copy is never
read (**INFERRED** unused). No draws.

### 7.3 The eight Flowmotion uses (all hold A = hold B = 0, no random)

| element : part (life, loop) | A | B | fade | output factor over the updates |
|---|---|---|---|---|
| ASK MLC0 : p12 (11, 3..4) | 1,1,1,1 | 1,1,1,0.5 | 8 | alpha 1 -> 0.5 over 8, back over 8, ... |
| COM BSS0 : p2 (11, 2..3) | 1,1,1,1 | 1,1,1,0.5 | 2 | alpha 1, 0.75, 0.5, 0.75, 1, ... (output after each update: 0.75, 0.5, 0.75, 1.0, ...) |
| COM BSS0 : p3 | 1,1,1,1 | 1,1,1,0.8 | 2 | alpha between 1 and 0.8 |
| COM BSS0 : p11 (10) | 1,1,1,1 | 0.5,0.7,1,0.5 | 3 | towards a blue tint at half alpha and back, period 6 |
| COM BSS0 : p12 (8) | 1,0.9,0.7,1 | 1,0.4,0.4,0.3 | 1 | alternates B, A, B, A every update (first update already at B) |
| BOS BSS0 : p2 | as COM p2 | | 2 | |
| BOS BSS0 : p5 | as COM p12 | | 1 | |

With both holds 0, e0 = 0 and e1 = e2 = fade: the phases are 1 (counter 0..fade-1) and 3
(fade..2fade-1); phases 0 and 2 never occur, so the first update starts the fade to B at once.

---

## 8. FecGravity (19)

Nothing above contradicts README 8.4. Its added acceleration lands in the same Posac work `+0x30`
that the spawn acceleration was added to (section 2.4), so both survive the Posac's key change.

---

## 9. Open points

1. **Time base.** Generate's interval logic, the first-frame rule and Colorblink's counter depend on
   how many updates run per FEP frame (README 13.5). Generate keeps its period in frames for dt < 1;
   Colorblink does not.
2. **Draw-side scale of MDL vertices** (5.4): whether the drawn vertex positions also use
   `s16 / 32768 * mesh[0x24]`. That belongs to the model topic; the generator's own reading is
   certain.
3. **The other three `0x5b23b0` callers** (`0x5808d0`, `0x581130`, `0x581210`) and argument 2 of
   `0x5b23b0` (scale by row lengths): other generator classes, not used by Flowmotion.
4. **Why five of part 16's slots have child -1** (3.4): authoring intent, not readable.
5. Parameter bytes with no reader: Generateanchor `+0x14`, Generate `+0x16` (non-zero in a few
   files outside Flowmotion).

## 10. Corrections and confirmations for README.md

- 8.3 Generateanchor: confirmed. Added: the speed direction is `normalize(P)` (from the emitter
  through the random cube point), zero with cube 0; types 1 and 9 add the emitter's Pos (and frame);
  a failed spawn ends the burst; **five** of AJC SHO0 part 16's nine blocks have child -1, not one.
- 8.3 Generate: the accumulator, rate, interval, chance and bit 12 are now read in full (4.3). The
  direction **is** normalised (`0x5b2591 -> 0x58c6c0`).
- 8.3 Generateanchorvertex: decoded (section 5). Its model is named by its own parameter `+0x0e`
  (section 3), the VTL by `+0x10`, the list by `+0x12`.
- 8.5 Rnd* tables: confirmed byte for byte. Mode 1 = [0, R], 2 = [-R, 0], other = [-R, R]; Rndscl
  with `+0x0c == 0` draws once; the value case writes output = new base (not output + d).
- 9.1: the MDL record's f32 `+0x00` and u32 `+0x14` **are read**, by Generateanchorvertex (scale and
  centring flag).
- 7.2 loop rewind: the init fallback gives t_new = loop start (not 0), and the stored "current entry"
  is the last entry examined (3.5 side note).

## 11. Tools

| tool | what |
|---|---|
| `tools/ddd_fep_generators_fep.py` | small stand-alone reader of bodies, parts, slots, init and key lists (file-relative offsets) used by the others |
| `tools/ddd_fep_generators_survey.py` | every generator block decoded with the layouts above (default: 19 elements), `--stats` (value histograms, 7 files and 667), `--loop` (block used after a wrap, emulating `0x5b80c0`), `--vtx` (Generateanchorvertex: model, list, vertex positions), `--mesh` (mesh position table check, 667 + resident), `--rnd` (Rnd* blocks), `--blink` (Colorblink blocks) |
| `tools/ddd_fep_generators_rngmap.py` | which command classes call the RNG, from which slot, how many calls (`--fm`, `--all-callers`) |

Run with Python 3 (`python -I`).
