# FEP commands: how DDD's effect interpreter runs them

Source: the Steam DDD exe (KH 2.8), read statically (image base 0x140000000; all addresses
below are RVAs). Data checks were run on the seven Flowmotion FEP files
(`effect/sys/ffa/bin/p_fa_{ajc0,akd0,ask0,com0}.fep`, `effect/sys/pcm/bin/p_ac_com0.fep`,
`effect/pc/ac/bin/p_fa_bos_0.fep`, `effect/wep/ex/bin/w_010_so.fep`: 46 element bodies,
355 parts). The container itself (header, sections, resources) is in `structure.md` /
`textures.md`; this file covers what the interpreter does with parts and commands.

Marking: a statement with an RVA and no tag was read in the code. **INFERRED** = follows
from code I read but I did not trace every path. **GUESS** = plausible, not shown.

Tools (all in `tools/`, each has a docstring):

| script | what it does |
|---|---|
| `ddd_fep_fn.py` | disassemble whole functions (bounds from `.pdata`, leaf functions by sweep), `-c FecX` all 9 slots of a class, `-x` callers |
| `ddd_fep_vtables.py` | every command id -> class -> 9 vtable slots |
| `ddd_fep_slotcalls.py` | every place the exe calls a command slot through the command table |
| `ddd_fep_parts.py` | parts / command records / init list / key blocks / parameter blocks of FEP files (`-s` summary, `-c Class` all params of one class) |
| `ddd_fep_check.py` | checks the structural claims below on all seven files (results quoted in section 9) |

---

## 1. The pieces the interpreter walks

The effect-creation code checks only header `+0x04` == 1 and `+0x06` == 9 (`0x5ac020`); the
4th magic byte is not "\0" in most files (ajc0 0, akd0 6, ask0 1, com0 7, p_ac_com0 6,
bos_0 1, w_010_so 1).
In the file every 32-bit offset is absolute (from the file start). At load, `0x5ab330` sets
header `+0x0C` bit 16 and calls `0x5aadd0` for every section-1 body; after that the code
treats every s32 offset below as **relative to the struct that holds it** (0 = null). Every
access in the interpreter is `base + s32[base + field]`. The u16 offsets (entry -> parameter
block) are relative to the list struct in the file too (they are never rebased).
(The rebase itself, `0x5aadd0`, was not read; the self-relative use was.)

### Element body (section 1, 0x34 bytes) - fields the interpreter uses
| off | type | use (RVA) |
|---|---|---|
| +0x00 | u16 | copied to effect instance `+0x04` (`0x5ac496`); bit 1 = the effect loops (tested `0x5b4424`, `0x5b44b0`); bit 0 = at the end wait for the parts to finish (`0x5b456f`) - the files have 0 or 2 |
| +0x04 | u32 | length in frames -> instance `+0x16C` (`0x5ac530`) |
| +0x0A | u8 | instance `+0x1B7`, kind counters (`0x5ac449`) |
| +0x0C | u32 | what a stop request (instance `+0x144` >= 0) does: 0 = fade `+0x148` from 1 to 0 over 30 frames then remove, 1 = the same over 15, 3 = ignore it and play on, other = remove at once (`0x5b43a7..0x5b4528`); the files have 3 or 0 |
| +0x10 | u16 | number of spawn events (`0x5b45f6`) |
| +0x14 | u16 | number of command ids in the +0x24 list |
| +0x1C | s32 | -> s32[] spawn events |
| +0x20 | s32 | -> s32[] parts (`0x5b741e`) |
| +0x24 | s32 | -> u16[] command ids used; for each, the file's per-id u32 (table at header `+0x08` -> {u32 count, u32 offset}) must equal the exe's (`0x9e7d10`, 158 entries, count at `0x9e7d00`), else the effect is not created (`0x5ac2c2..0x5ac2fa`). All seven files carry a table identical to the exe's. |
| +0x2C | s32 | -> s32[] part lists, used when a child-part index has bit 12 set (`0x5b0e80`) |

Spawn event: `s16 frame, s16 n, s32 -> u16 part index[n]` (`0x5b45d0..0x5b4693`).

### Part (0x48 bytes)
| off | type | meaning |
|---|---|---|
| +0x00 | u16 | part flags -> instance `+0x04`: bit 0 = wait for children before dying (`0x5b4542`), bit 1 = loopable (`0x5b441b`) |
| +0x02 | u16 | work size / 2 (`0x5b7446`: work bytes = 2*v) |
| +0x04 | u16 | lifetime in frames -> instance `+0x44` (`0x5b758a`) |
| +0x08 | u8 | number of commands (`ncmd`) |
| +0x09 | u8 | bit 0 set -> update pass 0, clear -> pass 1 (`0x5b750c`: group = ~b & 1). **All 355 Flowmotion parts have 2 (pass 1).** |
| +0x0B | u8 | matrix type, see 7.3 (instance `+0x90`, `0x5b766a`) |
| +0x0C | u16 | loop start frame (`0x5b6779`, `0x5b50fc`) |
| +0x0E | u16 | loop end frame (`0x5b443c`, `0x5b50c2`) |
| +0x13 | u8 | number of key blocks (`0x5b52dc`) |
| +0x14..+0x1B | u8[8] | counts of the 8 slot lists |
| +0x1C | s32 | -> s32[ncmd] command records |
| +0x20 | s32 | -> initial command list |
| +0x24 | s32 | -> s32[nkeys] key blocks |
| +0x28..+0x44 | s32[8] | -> u16[] slot lists (command indices), see 2.2 |

### Command record (one per command of a part)
`u16 id` (class: `[0x9e7450 + id*8]`, `docs/ddd-fec-commands.txt` line id-1), `u16 work/2`
(work byte offset = `(2*v) & 0xFFFF`, `0x5b5782..0x5b579c`), `s32 nref`, `s32 -> refs`.
refs = `u16 index[nref]` then `u16 work/2[nref]`: the other commands of the same part this
one reads, and where their work is. Commands read them through `ctx+0x88` (the current
record), e.g. FecLmatrix `0x58260f..0x582638`. Checked on all files: every ref's work offset
equals the referenced command's own (1968/1968).

### Command list = initial list and key blocks (same struct)
`s16 frame, u16 ?, s32 n, s32 -> entries[n]`; entry (8 bytes):

| off | type | meaning |
|---|---|---|
| +0 | u16 | command index in the part |
| +2 | u16 | parameter block offset, relative to the list struct |
| +4 | s16 | key start frame (local time = part time - this, `0x5b6027`) |
| +6 | s16 | key duration in frames: passed as "end time of the old key" to slot 1 (`0x5b523d`) |

Checked on all files: the initial list holds every command once, in order (355/355); key
blocks are sorted by frame and every entry's start equals its block's frame (3424/3424);
`+6` = next start of the same command - this start (3424/3424) and, for the last key,
lifetime - start (2273/2273; the 11 others are commands with no key at all, `+6` = 0).
The first key block is always at frame 0 and restates every command.

---

## 2. Dispatch: the nine vtable slots

Each command class is a static object whose only field is a vtable of **9 slots** (slot 9
of every vtable is the next class's RTTI locator). The default (base) implementations are
one-instruction stubs: `0x5aac00 0x5aac10 0x5aac40 0x5aac30 0x5aac60 0x5aac80 0x5aac70
0x5aac50` (slot 1..8; slot 5's default returns 0, the others return nothing). FecDummy has
its own stubs (`0x57fa40..0x57fac0`, slot 0 returns 1).

Before every call the dispatcher sets `ctx+0x78` = effect instance, `ctx+0x80` = part
instance, `ctx+0x88` = command record. "work" = part instance work area + the record's
work offset; "param" = the command's current parameter block (`partinst+0x68[i]`).

### 2.1 Slots (callee view: rcx = command object, rdx = ctx)
| slot | role | arguments after rcx, rdx | called from |
|---|---|---|---|
| 0 | **init** at spawn; returns 0 = abort spawn | r8 param (initial list), r9 work; `ctx+0x98` = spawn info (sec. 4) | `0x5b56f0` loop `0x5b5740..0x5b57cc`, call `0x5b57ba` |
| 1 | **key**: a new parameter block takes over | r8 new param, r9 work, [5] old param, [6] f32 old key's duration (or the current frame when the command had no key yet, `0x5b5243`) | `0x5b5180`, call `0x5b52bd` |
| 2 | **update** every frame | r8 param, xmm3 f32 local time (part time - key start), [5] work | `0x5b5f10` phase 2, call `0x5b61e3` |
| 3 | **draw** | r8 param, xmm3 local time, [5] work | `0x5b59d0` / `0x5b5c20` (calls `0x5b5b92`, `0x5b5dc0`) |
| 4 | **release** | r8 param, r9 work | part death `0x5b82b0` (call `0x5b83c6`); spawn abort `0x5b5857` |
| 5 | **loop rewind** | r8 param at loop start, xmm3 f32 its local time, [5] work, [6] old param, [7] f32 old local time | `0x5b6680`, call `0x5b6811` |
| 6 | **start**: once, on the part's first update pass, before keys | r8 param, r9 work | `0x5b5f10` phase 7 (`0x5b60f6`), inline in `0x5b53e0` (`0x5b5535`); guarded by instance flag 4 |
| 7 | after the pass-1 updates | r8 param, xmm3 local time, [5] work | `0x5b5f10` phase 8 (`0x5b604d`), driven by `0x5b5e70` |
| 8 | not called by the interpreter | - | no caller through the command table (`ddd_fep_slotcalls.py`); only Wave_tail_strategy / Wave_bindlocus_strategy override it - **INFERRED**: a hook their tail/polygon command calls directly |

In slot 2/3/7, when `ctx` bit 0 is set only FecLmatrix (id 14) and FecLmatrixbase (id 132)
are run (`0x5b61b8..0x5b61cb`).

### 2.2 The part's slot lists
The file precomputes, per part, which commands override which slot. List k is at
`part+0x28+4k`, its count at `part+0x14+k`:

| k | list off | count off | slot |
|---|---|---|---|
| 0 | +0x28 | +0x14 | 2 update |
| 1 | +0x2C | +0x15 | 3 draw |
| 2 | +0x30 | +0x16 | 4 release |
| 3 | +0x34 | +0x17 | 5 loop |
| 4 | +0x38 | +0x18 | - (empty in all 355 parts; no reader found) |
| 5 | +0x3C | +0x19 | 6 start |
| 6 | +0x40 | +0x1A | 7 |
| 7 | +0x44 | +0x1B | 8 |

Checked on all 355 parts: each list is exactly the set of the part's commands whose class
overrides that slot, in index order - except the draw list of two parts of
`p_ac_com0` element 5 (parts 3 and 5) which leaves out their FecMdlts_plus: those two
parts are never drawn. Slots 0 and 1 have no list: slot 0 runs for every command of the
initial list, slot 1 for every entry of a key block.

### 2.3 Init (`0x5b56f0`, called from the spawner `0x5b73d0` at `0x5b76d1`)
For each entry i of the initial list: `partinst+0x68[idx] = list + entry.param`, then
slot 0. If a slot 0 returns 0, slot 4 is called for entries 0..i and the part is freed
(`0x5b57e8..0x5b5863`, `0x5b7710`). Note: that clean-up loop passes record number i
(`0x5b5816`, edx = the failing index) for every entry, not each entry's own - it only
matters if commands are out of order, which never happens in the files.
`partinst+0x70[]` (current key entry) stays null until the first key.

---

## 3. Runtime structures

### 3.1 Context (`rcx` of every interpreter function; one global, saved at `0xaa3de0`)
`+0x00` flags (bit 0: no start slots and no keys, and only FecLmatrix/Lmatrixbase are
updated - `0x5b4741`, `0x5b61b8`), `+0x08` total time, `+0x10` dt, `+0x14` dt2,
`+0x18` allocator, `+0x2C` u16 mask of disabled effect categories (category = `attach[0] >> 28`),
`+0x38/+0x40` first/last effect instance, `+0x78/+0x80/+0x88` current effect instance /
part instance / command record, `+0x90` u8 current phase (0..0xC, see 5), `+0x98` spawn
info for slot 0, `+0xA0` number of key blocks processed in the current call,
`+0xA8`/`+0xB0` loaded 2D-animation / model tables.

### 3.2 Effect instance (one per played element; created by `0x5ac210`)
| off | meaning |
|---|---|
| +0x02 | u16 flags (0x20 = kill now `0x5b42ae`; 0x10 = stop looping) |
| +0x04 | u16 = body +0x00 (bit 1 loops) |
| +0x0C, +0x0E | u16 pause/hide flags (non-zero skips update `0x5b4326`, draw `0x5b5a1e`) |
| +0x10/+0x20/+0x30 | position / rotation / scale of the effect; `+0x60..+0x9F` its matrix, rebuilt each frame by `0x5ae260` from these and the attachment `+0x1A8` |
| +0x128 | next instance |
| +0x138 | f32 effect time; `+0x13C` this frame's dt; `+0x140` extra time added once; `+0x144` kill timer; `+0x148` fade factor |
| +0x168 | f32 start delay (counts down, effect frozen meanwhile, `0x5b5689`) |
| +0x16C | s32 length (body +4) |
| +0x170 | u16 serial for parts; `+0x172+2g` part count of pass g |
| +0x178+8g / +0x188+8g | first / last part instance of pass g (g = 0, 1) |
| +0x198 / +0x1A0 / +0x1A8 | file / element body / attachment |
| +0x1B0 | s16 next spawn event (-1 = all done) |
| +0x1B5 | s8 layer (the pass-1 update runs per layer) |

### 3.3 Part instance (what the commands write; built by `0x5b73d0`)
Allocated from `ctx+0x18` (`0x5b74fb`), tag byte 0x75. Header 0xA0 bytes, then the work
area, then two pointer arrays, an optional 0x30-byte draw state, then the matrices (16-aligned):

| off | meaning (RVA where set) |
|---|---|
| +0x00 | u8 0x75 |
| +0x02 | u16 state: 2 = has looped (`0x5b50ee`), 4 = start slot done, 8 = dead, 0x10 = loop wrap pending (`0x5b444c`), 0x40 (`0x5b7549`) |
| +0x04 | u16 part flags (part +0) |
| +0x06 | u16 serial (`0x5b7570`) |
| +0x08 | u8 pass (0/1) |
| +0x10 / +0x18 | prev / next in the pass list |
| +0x20 | parent part instance (null for parts spawned by events) |
| +0x28 | "frame owner": the instance whose `+0x88` matrix is this part's frame; null = the effect matrix (see 7.3) |
| +0x30 | u16 live children (`0x5b77b9`, `0x5b8411`) |
| +0x32 | u8 part index |
| +0x40 | f32 part time in frames (start value = spawn argument) |
| +0x44 | u16 lifetime |
| +0x46 | s16 next key block (-1 = none left) |
| +0x50 | draw state (0x30 bytes, `0x5bc440`) when the part has a draw list, else null |
| +0x58 | part definition |
| +0x60 | -> work area (= self + 0xA0) |
| +0x68 | -> param pointer per command |
| +0x70 | -> current key entry per command (null before the first key) |
| +0x78 | -> local matrix (written by FecLmatrix) |
| +0x80 | -> world matrix (written by FecLmatrix through `0x5b10e0`) |
| +0x88 | -> own frame matrix (types 2, 6..9 only) |
| +0x90 | u8 matrix type (part +0x0B) |
| +0x91 | u8 = FecLmatrix param byte 1 (`0x5821e9`) or the parent's (`0x5b12f3`) |

There is no fixed "particle" record with position/colour fields: **each command keeps its
state in its own slice of the work area**, the first vec4 of it being the command's output,
and the consumers find it through their refs. A FEP "particle" is a part instance
(children spawned by FecGenerate / FecGenerateanchor); FecMegaanm keeps its own
sub-particles (section 6.9).

---

## 4. Spawn info (`ctx+0x98`, read by slot 0)

`u32 flags` then pointers to vec4 at `+0x08 + 8*bit`. FecGenerate / FecGenerateanchor pass
flags 7 with position, velocity, acceleration (`0x581c43`, `0x580405`); spawn events pass
none.

| bit | ptr | used by |
|---|---|---|
| 0 | +0x08 position | Pos, Posmv replace the base; Posac **adds** to it (`0x582e6f`) |
| 1 | +0x10 velocity | Posmv replaces; Posac adds |
| 2 | +0x18 acceleration | Posac adds |
| 3 | +0x20 rotation | Rot, Rotmv, Rotac replace |
| 4 | +0x28 rotation velocity | Rotmv, Rotac replace |
| 5 | +0x30 rotation acceleration | Rotac replaces |
| 6 | +0x38 scale | Scl, Sclmv, Sclac replace |
| 7 | +0x40 scale velocity | Sclmv, Sclac |
| 8 | +0x48 scale acceleration | Sclac |
| 9 | +0x50 colour | Col, Colmv, Colac replace |
| 10 | +0x58 colour velocity | Colmv, Colac |
| 11 | +0x60 colour acceleration | Colac |

---

## 5. One frame

1. **Time** - task `0x554f80` (registered with priority 0xEA60 at `0x557eb1`): dt =
   `0x6294e0()*0.5`, dt2 = `0x540750()*0.5` (`0x554fbd`, `0x554fd4`), then `0x5b55d0`:
   for each effect instance, its dt is 0 if `+0x0A` is set, dt2 if `+0x0B` is set, else dt;
   0 while the start delay `+0x168` runs (`0x5b5670..0x5b56a7`). Then `0x5b4290(ctx, inst, dt)`:
   - `+0x13C` = dt (+ pending `+0x140`), `+0x138` += dt (`0x5b436d..0x5b439d`);
   - every part of both passes: time += dt (`0x5b4400`). If the part is loopable, the effect
     loops and is not stopping: time >= loop end -> state |= 0x10 (wrap pending). Otherwise
     lifetime <= int(time) and (no "wait for children" flag or no children left) -> the part
     dies: `0x5b82b0` calls slot 4 for its release list and unlinks it (`0x5b4535..0x5b456a`);
   - spawn events, only while effect time < length (and on the very first frame):
     while the next event's frame <= int(effect time), every part index of it is spawned
     with `0x5b73d0(parent = none, start time = effect time - event frame)`
     (`0x5b447d..0x5b44a6`, `0x5b45c2..0x5b4693`);
   - effect time >= length (`0x5b44ac..0x5b45ad`): a looping body (bit 1) ends only once it
     was told to stop (`+0x02` & 0x10) and both part lists are empty; otherwise body bit 0
     set -> ends when both part lists are empty, bit 0 clear -> ends at once. Ending =
     `0x5ad370(ctx, inst, 1)` (**INFERRED**: removes the instance with its parts).
2. **Pass 0 update** - task `0x554f70` (priority 0x249F0, `0x557ec7`) -> `0x5b53e0`, per
   effect instance: slot 6 for parts not started (phase 7), keys (`0x5b5040`), slot 2 (phase 2).
   No Flowmotion part is in pass 0.
3. **Pass 1 update** - from `0x557570` (reached from `0x5dffc0`): `0x5b47a0(ctx, layer)`
   (it also computes a camera-distance factor into `+0x4C` and may skip instances,
   `0x5b48e9..0x5b4d31`, **INFERRED** culling) or `0x5b4f70`: per instance of that
   layer `0x5ae260` (effect matrix from the attachment), then `0x5b4700`: phase 7 (slot 6),
   keys (`0x5b5040`), phase 2 (slot 2). Then `0x5b5e70(ctx, layer)`: phase 8 (slot 7).
   **INFERRED**: this runs after characters are posed, so parts follow bones the same frame.
4. **Draw** - `0x5b59d0` / `0x5b5c20(ctx, inst, part, pass)`: slot 3 for the draw list;
   `pass` picks commands whose byte in `0x9e7c60[id]` has bit 0 set or clear (only
   FecLensflare_light has it set); a command is skipped when `0x5e19b0()` >= 0x50
   (**GUESS**: a draw-queue limit).

The relative order of tasks 1 and 2 follows their priorities (**INFERRED** that lower runs
first).

### 5.1 Keys (`0x5b5040` -> `0x5b5180`)
Skipped when the instance's dt is 0. Per part (not dead): f = int(time). If a wrap is
pending: run keys up to loop end - 1, rewind (`0x5b6680`, below), clear 0x10, set 2,
time = (f - loopEnd) mod (loopEnd - loopStart) + loopStart + frac(time) (`0x5b50fa..0x5b512b`).
Then `0x5b5180(f)`: while the next key block's frame <= f, for each entry in order:
`old = +0x68[idx]`, `+0x68[idx] = new param`, `+0x70[idx] = entry`, slot 1(new, work, old,
t = old entry's duration, or f if there was none). Entry order matters (e.g. Rot before
Rndrot).

### 5.2 Loop rewind (`0x5b6680`)
`+0x46` = 0; for each command of list 3: find the key active at loop start (`0x5b80c0`),
make it current and call slot 5 with (new param, loopStart - key start, work, old param,
loopEnd - old key start); `+0x46` = the key after it.

### 5.3 Spawn (`0x5b73d0`) - order of events for one new part
allocate; header (3.3); zero `+0x68`/`+0x70`; init (2.3); link at the tail of its pass
list; parent's child count +1; frame owner / `+0x88` (7.3); if the current phase is 8 or 9
(spawned during the pass-1 update, i.e. by a generator) its keys up to int(time) are run at
once (`0x5b793c..0x5b7961`). Being linked at the tail, a generated child also gets its
slot 2 in the same pass; its slot 6 waits for the next frame's phase 7 (**INFERRED** from
the list walk `0x5b61f8`).

---

## 6. The commands

Units (read from constants): positions in PSP effect units, rotations in **degrees**
(converted with `256/360` and `2*pi/256`, `0x565a3d`), scale in **percent** (output = value
* 0.01), colour RGBA in **0..255** clamped (`0x78e78c` = 255; the files use 128 as "full",
**GUESS**: 128 = 1.0 as on PS2). Time t in frames of the FEP clock (dt per game frame
= half the game's delta, section 5).

### 6.1 The value / mv / ac families (Pos, Rot, Scl, Col)
Work layout shared by all twelve: `+0x00` output vec4, `+0x10` base, `+0x20` velocity,
`+0x30` acceleration (Pos-like = 0x20 used, mv 0x30, ac 0x40; the files reserve 0x40 /
0x60 / 0x80). Parameter blocks: value = 1 vec (+ for Pos* a byte at +0x0C); mv = value,
velocity at +0x10; ac = value, velocity +0x10, acceleration +0x20. Pos/Rot/Scl are vec3
(w ignored), Col is vec4.

| class (id) | slot 0 init | slot 1 key | slot 2 update | slot 5 loop |
|---|---|---|---|---|
| FecPos (1) | `0x583570` base = p; spawn bit 0 replaces; out = base | `0x583510` base += new - old; out = base | - | `0x5835c0` base = out |
| FecPosmv (2) | `0x583290` base = p, vel = v; bits 0,1 replace | `0x583150` base = (base + vel*t) - (old.p + old.v*t) + new.p; vel = vel - old.v + new.v | `0x583240` out = base + vel*t | `0x583300`: if new byte +0x0C == 1 and t_new == 0: re-init from param; else vel' = vel - old.v + new.v, base = base + vel*t_old - vel'*t_new; then slot 2 |
| FecPosac (3) | `0x582e10` base, vel, acc = params; bits 0,1,2 **add** | `0x582d60` -> `0x5afe70` (below) | `0x582d80` out = base + vel*t + 0.5*acc*t^2 | `0x582f90`: same reset test; else `0x5b04e0`; then slot 2 |
| FecRot (4) | `0x583d00` bit 3 | `0x583ca0` like Pos | - | `0x583d50` like Pos |
| FecRotmv (5) | `0x583a80` bits 3,4 | `0x583950` like Posmv | `0x583a40` | `0x583af0` like Posmv without the reset test, then wraps base into [-360, 360] |
| FecRotac (6) | `0x583760` bits 3,4,5 | `0x5836b0` -> `0x5afe70` | `0x5836e0` | `0x583800` `0x5b04e0`, wrap, slot 2 |
| FecScl (7) | `0x584320` bit 6; out = base*0.01 | `0x5842b0` | - | `0x5843a0` base = out*100 |
| FecSclmv (8) | `0x584120` bits 6,7 | `0x583fc0` | `0x5840d0` out = (base + vel*t)*0.01 | `0x5841c0` |
| FecSclac (9) | `0x583e60` bits 6,7,8 | `0x583d70` | `0x583dd0` out = (base + vel t + acc t^2/2)*0.01 | `0x583f30` |
| FecCol (15) | `0x57f950` bit 9; out = clamp(base) | `0x57f880` | - | `0x57fa00` |
| FecColmv (16) | `0x57f670` bits 9,10 | `0x57f470` (vec4) | `0x57f5b0` + clamp | `0x57f740` |
| FecColac (17) | `0x57f2f0` bits 9,10,11 | `0x57f1d0` -> `0x5b0090` (vec4) | `0x57f1f0` + clamp | `0x57f3e0` -> `0x5b0720` |

ac key change (`0x5afe70`, vec4 version `0x5b0090`), with t = old key's duration:
```
base' = (base + vel t + acc t^2/2) - (old.p + old.v t + old.a t^2/2) + new.p
vel'  = (vel + acc t) - (old.v + old.a t) + new.v
acc'  = acc - old.a + new.a
```
i.e. the deviation of the work from its parameters (random offsets, spawn velocities,
gravity) carries over and the motion restarts from the new block. With no deviation this
is simply "base, vel, acc = new block". Checked on data: consecutive key blocks are
authored continuous, e.g. `p_fa_ajc0` element 0 part 2 Sclac: 200 + 34*9 - 4.074*81/2 = 341.0
= the next block's value; part 1 Colac 160 - 23.74*3 + 8.713*9/2 = 128.0 = next block.

ac loop rewind (`0x5b04e0`, vec4 `0x5b0720`): vel' = vel(t_old) - old.v(t_old) + new.v,
acc' = acc - old.a + new.a, base' = pos(t_old) - (vel' t_new + acc' t_new^2/2): the part
continues from where it was.

The Pos byte `+0x0C` (reset on loop) is 0 in all 745 Pos/Posmv/Posac blocks of the files.
FecPos/Posmv/Posac also override slot 6 (`0x5835e0`, `0x583440`, `0x583080`): if the part
has no parent and its matrix type is 2, the base position (y, z negated) is transformed by
the effect matrix once at start (`0x5b21a0`) - type-2 parts are placed in world space.

### 6.2 FecLmatrix (14) - builds the part's matrices (used by 353 of 355 parts)
refs (always this order in the files): 0 = Pos*, 1 = Scl*, 2 = Rot*. Param (16 bytes):

| byte | meaning |
|---|---|
| 0 | attach mode; **equals the part's matrix type in all 706 blocks** (7.3) |
| 1 | -> instance `+0x91`; low 3 bits: 1 = FecMdl draws with a camera matrix (`0x589b18`, **INFERRED** billboard); 3 or 4 = for parts without a draw state, `0x5b1510` combines the matrix with the frame (7.2). Files: 0 (336), 1 (302), 2 (32), 3 (24), 11 (12) |
| 2 | 1 = orient along the Pos command's velocity (`0x5822fd`; Posmv vel, Posac vel + acc*t) - 0 in all files |
| 3 | rotation order 0..5 (below) - 0 in 700 of 706 |
| 4, 5 | passed to `0x5bc450` with the world position (`0x582590`; **GUESS**: sort/light group) |

slot 0 `0x5825c0` (init) and slot 2 `0x5821b0` (every frame): local `+0x78` = S * R * T where
S = diag(Scl output), R from the Rot output (degrees) and T = Pos output, **after negating
y and z of both position and rotation** (`0x5826a1..0x5826d4`). Matrices are row-vector
(v' = v M; `0x63ad40`), and `0x582990(order)` multiplies `M = S * Ra * Rb * Rc`:

| order | Ra, Rb, Rc |
|---|---|
| 0 | Z, Y, X |
| 1 | Z, X, Y |
| 2 | Y, Z, X |
| 3 | Y, X, Z |
| 4 | X, Z, Y |
| 5 | X, Y, Z |

with RotX (`0x565a20`) rows (1,0,0),(0,c,s),(0,-s,c); RotY (`0x55f3e0`) (c,0,-s),(0,1,0),(s,0,c);
RotZ (`0x565b50`) (c,s,0),(-s,c,0),(0,0,1); translation row = position. Orders > 5 leave S
without rotation or translation. Then `0x582770` -> `0x5b10e0(mode = byte 0)` makes the
world matrix `+0x80` (7.2). Slot 6 `0x582d30`: for a part with no parent, capture its frame
(`0x5b85a0`, same rules as the spawner). Slot 1 does nothing.

### 6.3 FecGenerate (18) - continuous emitter of child parts
refs: 0 = the part's Pos command (emission point). Work: f32 accumulator.
Param (24 bytes):

| off | type | meaning |
|---|---|---|
| +0x00 | f32 | rate: children per frame |
| +0x04 | f32 | half-size of the random position cube (each axis uniform in [-r, r], `0x581cff..0x581d7f`) |
| +0x08 | f32 | speed: child velocity = unit(offset) * this (`0x581e73`) |
| +0x0C | f32 | child acceleration = unit(offset) * this (`0x581e54`) |
| +0x10 | s16 | child part index (< 0 = off); bit 12 = index of a part list (`0x5b0e80`) |
| +0x12 | u16 | interval: frames to wait after an emission |
| +0x14 | s16 | must be >= 0 (`0x581a60`; all 0) |
| +0x17 | u8 | chance in % per child (all 100 in the files) |

slot 0 `0x582110` acc = 0. slot 1 `0x5819f0`: acc += old rate if another key block was
already run in this call (`ctx+0xA0` > 0), else acc = 0. slot 5 `0x582120` nothing.
slot 2 `0x581a20`, per frame with the instance dt:
```
if rate == 0 and acc == 0 or dt == 0: return
steps = dt  (near the end of a non-loopable part's life: the frames left, counted in intervals)
total = rate*steps + acc
threshold = interval > 0 ? 1 : rate
if total < threshold: acc = total; return
if dt < 1: total = rate
n = int(total); acc = total - n - interval*rate
spawn n children (each with the % chance)
```
(the first-frame special case `0x581b82..0x581bb9` sets acc = 1 - dt when rate is 1 and the
part just started.) Each child: offset = random cube; position/direction by
`0x5b23b0(child matrix type)`: types 0 and 2..8 keep the offset as the position (the
emitter's transform reaches them later through FecLmatrix's parent step, 7.2 - **INFERRED**),
type 1 adds the emitter's Pos output at spawn (identity matrix + that translation), type 9
goes to world space; velocity and acceleration as above; y and z of all three negated;
spawned with `0x5b7980` (parent = this part, start time 0, spawn info flags 7).
**INFERRED**: the direction is normalised in the tail of `0x5b23b0` (`0x5b2584..`, not read).

### 6.4 FecGenerateanchor (151) - one burst per key (191 uses in the files)
Child spawning as FecGenerate (`0x580350`): +0x00 = burst count (truncated, `0x580435`),
+0x04 cube, +0x08 speed, +0x0C acceleration, +0x10 child index (< 0 = off), +0x12 must be
>= 0 (`0x580383`); no rate/interval/chance logic was seen. In the files +0x00 is 1 (372 of
436 blocks) or 0, +0x04 0 or 0.3. Work: u8 pending. slot 0 `0x580340` pending = 0; slot 1 `0x5802d0`: if a burst was
pending, fire it with the old block, then pending = 1; slot 2 `0x580310`: if pending, fire
with the current block, pending = 0; slot 5 `0x580890` pending = 1. So every key block
naming it (and every loop) emits one burst on the next update.

### 6.5 FecGravity (19)
ref 0 = a Posac. Param: s16 g. Only slot 1 `0x585350`: ref's acceleration.y (`work+0x34`)
+= g/100 * 0.098. Acts at each key block naming it; the Posac's own key change keeps it
(its acc' = acc - old.a + new.a). Both uses in the files have g = 10. FEP +y becomes engine
-y (FecLmatrix negates y and z), so positive g pulls down (**INFERRED**: FEP y points down).
Note FecPosac's slot 1 does not refresh its output (`0x582d60`); the next slot 2 does.

### 6.6 FecRndpos (22), FecRndrot (23), FecRndscl (24), FecRndcol (25)
ref 0 = the Pos* / Rot* / Scl* / Col* command to perturb. Slot 1 only (`0x5a9510`,
`0x5a9ea0`, `0x5aa800`, `0x5a8a80`), **skipped once the part has looped** (state bit 2).
For k = value, velocity, acceleration: if enabled, d = random per component by mode, then
value: base += d and output += d (Rndscl: output = base*0.01); velocity: `+0x20` += d;
acceleration: `+0x30` += d. Modes: 0 -> [-r, r], 1 -> [0, r], 2 -> [-r, 0]. Random =
`0x613020()>>1 * 2^-31`.

| class | ranges | enable bytes | mode bytes | other |
|---|---|---|---|---|
| Rndpos, Rndrot (tables `0x9e830c..`, `0x9e831c..`) | vec3 at +0x00, +0x10, +0x20 | +0x0C, +0x0E, +0x1C | +0x0D, +0x0F, +0x1D | |
| Rndscl (`0x9e832c..`) | +0x00, +0x10, +0x20 | +0x0D, +0x0F, +0x1D | +0x0E, +0x1C, +0x1E | +0x0C: 0 = one random for all axes (from range.x), else per axis |
| Rndcol (`0x9e82fc..`) | vec4 at +0x00, +0x10, +0x20 | +0x30, +0x32, +0x34 | +0x31, +0x33, +0x35 | no clamp here |

FecRndpos1 (31, unused in the Flowmotion files): slot 1 `0x5a8fc0`, one vec3 range at
+0x00, per-axis mode bytes +0x0D..+0x0F (0 [-r,r], 1 [0,r], 2 [-r,0], 3 none), target by
byte +0x0C (0 value, 1 velocity, 2 acceleration), sets w = 1.

### 6.7 FecAnm (10) / FecAnm_plus (152) - 2D animation clock (Anm_plus: 100 uses)
refs (files): Colac/Col, Lmatrix. Work: `+0` f32 time, `+4` u16 frame (0xFFFF = ended),
`+6` u8 flip bits. Param: `+0x0C` s32 2D-animation index (header section via `+0x38`),
`+0x10` f32 speed, `+0x18` u16 random start, `+0x1A` u8 flip mode.
slot 0 (`0x586720`, plus `0x586dc0`): time = -dt + rand*u16[0x18]; flip: mode 1 -> bit 0
random, 2 -> bit 1 random, 3 -> 0..3. slot 2 (`0x5865f0`, plus `0x586c80`): unless ended:
time += instance dt; t = time*speed; past the animation's length: if its loop flag
(`+0x10` & 0x80) time = int(t) mod length (scaled value stored back - a quirk), else
frame = 0xFFFF; frame = index of the key whose summed durations pass t (12-byte keys at
anim `+0x2C`, s32 duration). Slot 3 (`0x586870`, plus `0x586f10`) draws: **not decoded**.

### 6.8 FecMdl (11) / Mdl_plus (154) / Mdlts_plus (155) - models
FecMdl: slot 0 returns 1, slot 1 nothing, slot 3 `0x5899a0` draws model `param+0x0C`
(header section via `+0x34`; skipped if its handle `+0x10` is -1) starting from the part's
local matrix `+0x78` (`0x589ad3`), with a camera matrix when the Lmatrix byte 1 & 7 == 1
(**INFERRED** billboard); also reads `+0x10` f32, `+0x15`,
`+0x18`, `+0x19` bytes (`0x589e63..0x58a058`), meaning **not decoded**.
FecMdlts_plus (85 uses): slot 0 `0x588f60` copies param `+0x10..+0x27` into work (three
vec2: value, velocity, acceleration), slot 1 `0x588c80` rebases them like the ac family and
(if byte `+0x4A` and the part has not looped) adds randoms from ranges at `+0x28...`;
**GUESS** ("ts"): a texture scroll. Draw slots not decoded.

### 6.9 FecMegaanm (21) / Megaanm_plus (153) - sub-particle system (25 uses)
slot 0 `0x58bc30` allocates `0x30 + 0x80*N` bytes, N = param u16 `+0xF0` (work `+0` =
pointer; slot 4 `0x58bce0` frees). slot 1 `0x58a940` sets the buffer's fade-in/out factors
from param `+0x2C`, `+0x3C`, `+0xF2` (particle life), `+0x110..+0x112`. The per-particle
update (slot 2 `0x58aad0`, ~1000 instructions) and draw are **not decoded yet**: this is the
next step for the sparkle effects.

---

## 7. Matrices and attachment

### 7.1 Effect matrix
`0x5ae260` each pass-1 frame: effect `+0x60` from its position/rotation/scale and the
attachment `+0x1A8` (the bone it was started on; see the ESE/EAD notes).

### 7.2 World matrix of a part (`0x5b10e0`, mode = Lmatrix byte 0)
The local matrix `+0x78` is scaled by the passed factor (1.0) and copied to the world one
`+0x80` (`0x5b1112..0x5b11d6`), then, by mode, when the part has a parent:
mode 0 - the translation rows of both are transformed by the parent's local matrix
(`0x5b13e5..0x5b1488`); mode 3 - local = local * parent local, world = world * parent world
(`0x58c5b0`, `0x5b131e..0x5b1359`); mode 4 - via `0x5b2d00`/`0x5b2de0` with the parent's
matrices, then the translations (`0x5b1202..0x5b12e3`; **INFERRED**: parent position without
its rotation); modes 1, 2, >= 5 - nothing (`0x5b11fc`). The part also takes the parent's
`+0x91`. Then, only for a part **without** a draw state (`+0x50` null) and Lmatrix byte 1 & 7
= 3 or 4, `0x5b1510` combines the matrix with the frame (owner's `+0x88`, else the live
effect matrix `+0x60`, `0x5b152e..0x5b154e`). Drawn parts keep the local matrix and the
draw command starts from it (FecMdl `0x589ad3`): **INFERRED** the frame is applied at draw
time. Not every branch of `0x5b1510` was followed.

### 7.3 Part matrix types (part `+0x0B` = Lmatrix byte 0), `0x5b77bd..0x5b7935`, `0x5b85a0`
| type | frame | files |
|---|---|---|
| 0, 1, 3, 4, 5 | none of its own: follows its parent's owner, else the **live** effect matrix | 0: 52, 1: 159, 3: 127, 4: 4 |
| 2 | own `+0x88` captured at spawn/start from the parent's (or effect) frame, rotation reset to identity: world position snapshot | 0 |
| 6, 8 | own frame captured at spawn/start (full) | 6: 11 |
| 7 | as 6 with the axes normalised (no scale) | 0 |
| 9 | identity (world origin) | 0 |
| 0xFF | no matrices at all | 2 |

---

## 8. What the Flowmotion files use

From `ddd_fep_check.py` / `ddd_fep_parts.py -s -flow` (count = commands over 355 parts):
Lmatrix 353, Rot 285, Colac 253, Generateanchor 191, Posac 187, Scl 187, Pos 167, Sclac 166,
Anm_plus 100, Mdlts_plus 85, Mdl_plus 55, Rndrot 52, Rotac 40, Col 28, Rotmv 28,
Megaanm_plus 25, Generate 12, Wave_bindlocus_strategy 10, Bindlocus_polygon 10,
Colorblink(_parameter) 8 each, Rndpos 6, Generateanchorvertex 5, Rndscl 5,
Null_routegenerator 3, Locus_tail_strategy 3, Shapetail 3, Rndcol 2, Gravity 2, Posmv 1,
Ellipse_line_particle 1, Radialg_line_particle 1, Flash 1, Mask 1.
**Not used**: Sclmv, Colmv, Rndpos1, Anm, Megaanm, Mdl (only their `_plus` forms).

Typical part: `Pos*/Rot*/Scl*/Col*` -> `Lmatrix(Pos, Scl, Rot)` -> a draw command
`(Col*, Lmatrix)`; emitters reference the Pos command; Rnd* reference the command they
perturb.

## 9. Checks run (`tools/ddd_fep_check.py`, all seven files)
init list = all commands in order 355/355; key blocks sorted 355/355; key entry start =
block frame 3424/3424; entry +6 = duration 3424/3424 (+2273 last keys = life - start);
refs' work offsets match 1968/1968; work offsets inside the part's work size 355/355;
list k <-> slot (2,3,4,5,-,6,7,8) 355/355 for every list except the draw list (353/355,
the two p_ac_com0 parts above); list 4 empty everywhere.

## 10. Open
- Slot 8 has no caller through the table; who calls the wave strategies' slot 8.
- Part list 4 (`+0x38`/`+0x18`): always empty, no reader found.
- Draw slots (Anm/Mdl/Mdlts/Megaanm/tails/polygons) and the per-particle Megaanm update.
- The game delta from `0x6294e0`: **GUESS** 1.0 per 60 fps frame, which makes FEP time
  30 frames per second.
- Part matrix types 0, 1, 3, 4, 5 differ somewhere outside the code read here (draw/cull).
- `0x5aadd0` (rebase at load) was located but not read.
