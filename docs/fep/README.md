# FEP: DDD's visual effect format (consolidated spec)

Kingdom Hearts Dream Drop Distance, Steam build inside KH 2.8: `effect/**/*.fep` (+ `.ese`,
`effect/resident/bin/resident.frr`). Engine namespace `FunnyEffectsForPSP`. Everything here was
read from the files and the exe (image base 0x140000000; **every address is an RVA**); the game was
never run.

This file merges the work of three decoders and the three skeptics who re-checked them. The topic
specs hold the full detail and the decoders' reasoning; **where a topic spec disagrees with this
file, this file is the corrected version** (section 14 lists every correction):

- [structure.md](structure.md): container, header, sections, bodies, parts, command lists, relocation.
- [commands.md](commands.md): the interpreter (vtable slots, frame order, runtime structs) and the
  command classes decoded so far (Pos/Rot/Scl/Col families, Lmatrix, generators, Gravity, Rnd*).
- [textures.md](textures.md): sections 2..5 (TEX/MDL/ANM/VTL), resident.frr, texture decoding.

**Labels.** Unmarked = checked: in the exe at the RVA given and/or on the data of **all seven**
Flowmotion FEPs (`sys/ffa/bin/p_fa_ajc0`, `p_fa_akd0`, `p_fa_ask0`, `p_fa_com0`,
`sys/pcm/bin/p_ac_com0`, `pc/ac/bin/p_fa_bos_0`, `wep/ex/bin/w_010_so`); "667" = also checked on
every FEP of the game. **INFERRED** = follows from checked facts, not seen directly. **GUESS** = a
guess. **Decoder only** = stated by a decoder with RVAs but not re-verified by a skeptic.
**Uncertain** / **Corrected** = the skeptic's verdict.

Contents: 1 loading - 2 header - 3 signature table - 4 names, groups, elements - 5 bodies -
6 parts, slots, command lists - 7 the interpreter - 8 commands - 9 resources - 10 ESE -
11 Flowmotion: command classes per element - 12 the dive and the slam, part by part -
13 unknowns, in priority order - 14 corrections to the topic specs - 15 tools.

---

## 1. Loading

1. FEP is resource type 10 (extension table entry `0x9ea180` = `{0x800a, "fep"}`; ESE is type 13,
   `0x9ea1b0`). The post-load switch `0x6169d1` (jump table `0x616e18`, index bytes `0x616e80`)
   sends type 10 to `0x616cce -> 0x558300 -> 0x5ab930` (effect system object `[0xaa3cd0]`).
2. `0x5ab930` checks the bytes `F`,`E`,`P` only (`0x5ab93a..0x5ab953`; byte 3 is 0..65 over the 667
   files), s16 `+4 == [0x7ed6a8]` (1) and s16 `+6 == [0x7ed6ac]` (9) (`0x5ab959`, `0x5ab965`);
   does `and word [+0x18], 0xf` (`0x5ab971`); `owner = ++[sys+0x30]` (`0x5ab976`); relocates
   (`0x5ab330`); registers the resources of sections 2, 4, 3, 5 in that order
   (`0x5ab993..0x5ab9bd`, section 9).
3. **Offsets.** In the file every 32-bit offset counts from the file start. `0x5ab330` sets bits
   16..23 of header `+0x0c` and relocates only if bits 16 and 18 were both clear (`0x5ab33c`,
   `0x5ab39b`): it calls `0x5aadd0(body, file)` for every section-1 body, which adds `file - owner` to every
   **non-zero** offset under section 1 (`0x5aadd0..0x5ab325`, read in full; zero stays "none",
   `0x5aae12`, `0x5aaea6`). After loading, an offset field is relative to the struct that holds it,
   and **the entries of an offset table are relative to the struct that owns the table** (body or
   part), not to the table (`0x5b46e0`, `0x5b51c3..0x5b51d8`, `0x5b741e..0x5b7431`). A spawn
   event's `+4` becomes relative to the event. The header, the section tables and sections 0, 2..7
   are never relocated; code adds the file base to them (`0x5ac274`, `0x5abb83`, `0x5acc75`,
   `0x5b3053`). The u16 parameter offsets of command entries are list-relative in the file already.
   Over 667 files no relocated struct is reached twice and none points at itself (skeptic).
   A port that reads the file directly uses file-relative offsets everywhere.
4. **Starting an element by name** (e.g. `0xfc305`: `sprintf("%s.fep")` `0x64ce0`, lookup
   `0x6186f0`): `0x5567d0` gets the FEP and ESE references, `0x5abb70` finds the name in section 7,
   `0x5559a0 -> 0x54fc80 -> 0x5acb40` turns its id into a group and element, `0x5ac210` creates the
   element instance. Creation fails when any command id the body lists has a signature-table entry
   that differs from the exe's (`0x5ac2c2..0x5ac30a`; a second copy of this check is at
   `0x5af620`, compare `0x5af696`) or when the start frame is at or past the body's length
   (`0x5ac30c`).

## 2. Header (0x48 bytes)

| off | type | meaning |
|---|---|---|
| 0x00 | char[3] + u8 | `"FEP"` + a byte not checked (0..65; **GUESS** a save counter) |
| 0x04 | s16 | 1 (version, must be 1) |
| 0x06 | s16 | 9 (version, must be 9) |
| 0x08 | u32 | 0x48 = offset of the command signature table header (section 3) |
| 0x0c | u32 | 0 in the file; runtime flags (bits 16..23 set by `0x5ab330`) |
| 0x10 | u32 | **id base**; bits 0-16 are 0 (667); returned by `0x5aadc0` |
| 0x14 | u32 | file size (667) |
| 0x18 | u16 | category = id base >> 28 exactly (667); masked to 4 bits at `0x5ab971`. ajc0/akd0/ask0/bos_0 3, com0/p_ac_com0 2, w_010_so 4 |
| 0x1a | u16 | 0 (667); no reader found |
| 0x1c..0x26 | u16 x6 | counts of sections 0 (groups), 1 (bodies), 2 (TEX), 3 (MDL), 4 (ANM), 5 (VTL) |
| 0x28..0x44 | u32 x8 | offsets of sections 0..7; 0..5 point to u32[count] tables of file offsets; 6 = `{u16 count, u16 garbage, u32 -> u32[count]}`; 7 = `{s32 count, u32 -> 12-byte records}` |

## 3. Command signature table (header +0x08)

`{u32 158, u32 offset}` at 0x48, then 158 x 8 bytes, one per command id 0..157:
`u8 a, u8 b, u16 c, u16 param_size, u16 work_size (2-byte units)`. **Byte-identical to the exe's
table at `0x9e7d10`** (count `[0x9e7d00]` = 158) in 667/667 files. `0x5ac2f2` compares
`u32[file_table + id*4]` with the exe's for each id the body uses (ids >= 158 fail, `0x5ac2e8`).

- **No exe code reads param_size or work_size**: the only references to the table are the two
  integrity checks above. Their meanings rest on the data, which fits them exactly: parameter
  blocks sized by param_size never overlap any other structure in 667 files; work areas sized by
  work_size never overlap inside a part, and `work offset + work_size <= part +0x02` (239364/239364).
- Ids 152..157 (`_plus` classes, 156, `FecMaskModel`) have all-zero records. **Corrected**: a
  `_plus` parameter block is the base class's layout (FecAnm 0x1f / FecMegaanm 0x115 / FecMdl 0x1a /
  FecMdlts 0x4b bytes) **plus at least one trailing field**: a non-zero value right past the base
  size in every Anm_plus block (22226/22226, values 1..4), Megaanm_plus (16428/16428), Mdl_plus
  (6042/6070) and Mdlts_plus (13761/13773); the base classes never have it. Its meaning is unknown.
  That the leading bytes keep the base meaning is **INFERRED** (the draw slots read `+0x0C` as the
  resource index, as the base classes do: section 8.6).
- FecLocus_tail_strategy (68), FecShapetail (80), FecDistortional_model (119) carry non-zero bytes
  past param_size (940 / 689 / 165 blocks over 667 files; e.g. a u32 1 at `+0x28` of the Locus_tail
  blocks in p_fa_com0); whether the engine reads them is unknown.
- a, b, c: unknown.

## 4. Names (section 7), groups (section 0), elements

**Element id** = `id base | group << 10 | element`: bits 0-8 element, bit 9 "special" (looked up in
section 6, empty in 667 files; its record layout and handler `0x5b6f30` were not decoded), bits
10-16 group, bits 28-31 category. Two code paths test the category against the manager mask
`+0x2c` **differently** (**uncertain** which is meant): the time step does `test mask, id>>28`
(`0x5b4311..0x5b4324`, a value AND) and the pass-1 update does `bt mask, id>>28`
(`0x5b4717..0x5b4726`, a bit index). Either way a set match skips the element.

**Name record** (section 7), 12 bytes: `u32 element id (-1 = none), u32 length, u32 offset of the
NUL-terminated name`. `0x5abb70` compares length then text and returns the record index. Every name
resolves and every element has a name (4145/4145).

**Group record** (section 0), 12 bytes: `u32 group id (low 10 bits clear), u16 body index, u16
element count, u32 -> u32[n] element offsets`. `0x5acb40` matches `(id >> 10) & 0x7f`
(`0x5acc96`), then `id & 0x1ff` (`0x5accfc`), and uses body = `+4` (`0x5acd39`). In all 667 files
every group has exactly one element and every body one group.

**Element** (0x9c bytes; instance `+0x1a8` points to it). Fields read by the code (decoder only,
except where a check is cited):

| off | meaning |
|---|---|
| 0x00 | element id |
| 0x06 | u8 view-angle test (`0x5b4a0b..0x5b4f66`): 0 off; 2/3 compare the camera direction with an axis (3 rotates it by `+0x60..+0x68`), thresholds `+0x74`, `+0x84`. 0 in all 42 Flowmotion elements |
| 0x07 | u8 -> instance `+0x1bc` |
| 0x0a | u8 view-bounds test (`0x5b4903`): 0 built-in bounds, 2/3 extent `+0x40` / centre `+0x50`. 2 in all 42 |
| 0x0b | u8 distance test (`0x5b49c2`): 0 = 124/240, 4 = 70/115, 2/3 = `+0x94`/`+0x98`, 1 = none |
| 0x10 | vec4 added to the start position (`0x5ac67b..0x5ac6ce`); (0, -0, -0) in the dive and slam elements |
| 0x30 | vec4, no reader seen; (1,1,1) in 40 of 42, 1.5 / 1.2 in two p_fa_com0 elements (**GUESS** scale) |
| 0x8e | u8 -> instance `+0x1be` |
| 0x90 | u16 start delay: the instance clock starts at minus this (`0x5ac4eb..0x5ac501`); 0 in the dive and slam elements |

Not understood: `+0x04`, `+0x08`, `+0x0c`, `+0x20`, `+0x92`. Vec4 `w` words hold authoring garbage.

## 5. Bodies (section 1, 0x34 bytes)

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | flags -> instance `+0x04` (`0x5ac496`). When the clock reaches the length (`0x5b44ac..0x5b45ad`): bit 1 = looping effect (its loopable parts loop, `0x5b4424`; it ends only once told to stop and empty); bit 0 = ends when no parts are left (`0x5b456f`); neither = ends at once. Ending = `0x5ad370` (**INFERRED** removes the instance) |
| 0x04 | s32 | length in frames -> instance `+0x16c` (`0x5ac530`) |
| 0x0a | u8 | -> instance `+0x1b7`, picks a manager counter (`0x5ac449`) |
| 0x0c | u32 | stop request (instance `+0x144 >= 0`, initialised to -1.0 at `0x5ac520`), `0x5b43a7..0x5b43bf`: 0 = fade `+0x148` over 30 frames (`[0x867efc]`) then remove; 1 = over 15 (`[0x867ef0]`); **3 = ignore and keep running**; other = remove at once (`0x5b4518`). Game-wide 3:3043, 0:943, 1:85, 2:74 |
| 0x10..0x1a | u16 x6 | counts: spawn events, parts, command ids, 12 (always), part groups, `+0x30` records (always 0) |
| 0x1c | s32 | -> s32[] spawn events |
| 0x20 | s32 | -> s32[] parts; **an entry may be 0 = no part** (`0x5b7428`; 17 in 8 non-Flowmotion files) |
| 0x24 | s32 | -> u16[] command ids used (every slot id of every part, no duplicates; 4145/4145); checked at creation |
| 0x28 | s32 | -> 12 records `{u32 count, s32 -> u16[count]}`, relocated (`0x5ab0b2..0x5ab227`); no reader found |
| 0x2c | s32 | -> part groups (below) |
| 0x30 | s32 | records with an offset at `+8` (relocated, `0x5ab2ac`); never present |

Not understood: `+0x02`, `+0x08` (0..240, **GUESS** frames), `+0x0b`.

**Spawn event**: `s16 frame, s16 n, s32 -> u16 part ref[n]`. While the element clock < length (and on
the first frame), every event whose frame <= int(clock) spawns its parts as roots with start time =
clock - frame (`0x5b45b2..0x5b4693 -> 0x5b73d0`; spawn info null, `0x5b465c`). The loop stops at the
first future event (`0x5b460c`), so **events must be in ascending frame order** (they are: 4145/4145).

**Part ref** (u16): bits 0-11 = part index (`& 0xfff` at `0x5b740b`); bit 13 sets parent state
`|= 0x80` and child state `|= 0x40` (`0x5b752d..0x5b7549`, meaning unknown); bit 12 (generator
parameters only) = a part-group index (`bt 0xc` at `0x5b0e85`). 26 spawn refs game-wide point at
empty part entries, which the spawner skips.

**Part group** (`+0x2c`): `u16 count, u16 garbage, s32 -> {s16, u16 ref}[count]`; `0x5b0e80` returns
the count, `0x5b0ec0` record j's ref. In every file each body has one group listing its parts by
**file-wide** number (37489 records), which would overrun every body's table but the first; no
generator parameter in 667 files has bit 12 set and every non-negative generator ref is < its
body's part count (45602/45602). **INFERRED**: authoring data the game does not use. (Note:
`0x5800a7..0x5800b9` passes only the low byte of the ref to `0x5b0ec0`.)

## 6. Parts, slots and command lists

### 6.1 Part (0x48 bytes)

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | flags -> instance `+0x04`: bit 0 = after its life, live on while it has children (`0x5b4542`); bit 1 = loops between `+0x0c` and `+0x0e` when the body loops (`0x5b441b`, `0x5b5040`) |
| 0x02 | s16 | work size / 2 (read signed, `0x5b7446`); work at instance `+0xa0` |
| 0x04 | u16 | lifetime in frames -> instance `+0x44` (`0x5b758a`) |
| 0x06 | u16 | its own index (37328/37328) |
| 0x08 | u8 | slot count; a part with 0 slots is not spawned (`0x5b743a`) |
| 0x09 | u8 | bit 0 picks the element's part list: set -> `+0x178` (pass 0, updated by `0x5b53e0`), clear -> `+0x180` (pass 1, `0x5b4700`) (`0x5b750c`). **2 in every part of every readable file**: pass 0 is unused |
| 0x0a | u8 | 0 (37328/37328) |
| 0x0b | u8 | matrix type -> instance `+0x90` (`0x5b766a`): 2 and 6..9 get three 0x40-byte matrices, 0xff none, others two (`0x5b749f..0x5b74e6`) |
| 0x0c / 0x0e | u16 | loop start / end frame (`0x5b6779`, `0x5b443c`, `0x5b50c2`) |
| 0x10 | u8 | no reader found. **Corrected**: non-zero in 3 Flowmotion parts (p_fa_akd0 body 0 parts 4, 9 = 3; p_fa_com0 body 18 part 10 = 4) and 2912 parts game-wide |
| 0x11 | u8 | = lifetime & 0xff (37328/37328); no reader found |
| 0x12 | u8 | **1 in all 37328 parts**; no reader found |
| 0x13 | u8 | key-list count (`0x5b52dc`) |
| 0x14..0x1b | u8 x8 | counts of the 8 slot lists. `+0x15` (draw count) also decides whether a 0x30-byte draw state is allocated at instance `+0x50` (`0x5b7478..0x5b75b4`, init `0x5bc440`) |
| 0x1c | s32 | -> s32[slot count] slot descriptors |
| 0x20 | s32 | -> init list |
| 0x24 | s32 | -> s32[key count] key lists |
| 0x28..0x44 | s32 x8 | -> u16[] slot lists (6.4) |

### 6.2 Slot descriptor (a "command" of the part)

`u16 command id` (class object `[0x9e7450 + id*8]`, `0x5b57b3`; names in
`docs/ddd-fec-commands.txt` line id-1), `u16 work offset / 2` (work = instance work +
`(2*v) & 0xffff`, `0x5b5782..0x5b57a4`), `u16 n + u16 0` (input count), `s32 -> u16 slot[n],
u16 work[n]` (the other slots it reads and their work offsets). Over 667 files every id is in
1..157 and maps to a real class (never FecDummy, 239364/239364), every input's work offset equals
that slot's own (208912/208912) and every input is an **earlier** slot. Commands read their inputs
through the current command record `ctx+0x88` (e.g. FecLmatrix `0x58260f..0x582638`, FecGravity
`0x585366`).

### 6.3 Init list and key lists

Same layout: `{s16 frame, s16 smallest span, s32 count, s32 -> entries}`, entries right after the
12-byte header, parameter blocks after the entries (147852 lists). Entry (8 bytes):
`u16 slot index, u16 parameter offset from the list header, s16 start frame (= the list's frame),
s16 span` (span = the same slot's next key frame, else the part's lifetime, minus the start:
329749/329749).

- The init list holds one entry per slot, in slot order, frame 0, span 0 (37328/37328).
- The first key list is always at frame 0 (37328/37328). **Corrected**: it does **not** always
  restate every slot (2386 parts game-wide; 11 of the 355 Flowmotion parts): a slot with no key at
  all (Colorblink, Null_routegenerator in the files) keeps `+0x70` null for life, its local time is
  the part time and its key method is never called.
- Parameter blocks are usually but not always 16-aligned (p_fa_com0 `0xba26`); use the offsets.
- 18994 key lists game-wide (235 in Flowmotion) lie at frame >= the part's lifetime; whether they
  ever run depends on the order of the death check (`0x5b4535`) and the key runner (`0x5b5180`), not
  established.

### 6.4 The eight per-part slot lists = the phases

Each command class is a static object holding a vtable of 9 methods (slot k at `+8k`). List k of a
part names the slots whose class takes part in phase k:

| list | count | vtable slot | phase | called by (RVA) |
|---|---|---|---|---|
| (init list) | | 0 `+0x00` | init at spawn; returning 0 aborts the spawn | `0x5b56f0` (`0x5b57ba`) |
| (key lists) | | 1 `+0x08` | key: a new parameter block takes over | `0x5b5180` (`0x5b52bd`) |
| `+0x28` | `+0x14` | 2 `+0x10` | update, every frame | `0x5b5f10` phase 2 (`0x5b61e3`) |
| `+0x2c` | `+0x15` | 3 `+0x18` | draw | `0x5b59d0`, `0x5b5c20` (`0x5b5b92`, `0x5b5dc0`) |
| `+0x30` | `+0x16` | 4 `+0x20` | release at part death | `0x5b82b0` (`0x5b83c6`); spawn abort `0x5b5857` |
| `+0x34` | `+0x17` | 5 `+0x28` | loop rewind | `0x5b6680` (`0x5b6811`) |
| `+0x38` | `+0x18` | - | none; empty in 37328/37328 parts; no reader | - |
| `+0x3c` | `+0x19` | 6 `+0x30` | first update only (instance state 4) | `0x5b5f10` phase 7 (`0x5b60f6`), `0x5b53e0` (`0x5b5535`) |
| `+0x40` | `+0x1a` | 7 `+0x38` | after all pass-1 updates | `0x5b5f10` phase 8 (`0x5b604d`) from `0x5b5e70` |
| `+0x44` | `+0x1b` | 8 `+0x40` | no call through the command table anywhere in `0x57f000..0x5bd000` | only Wave_tail_strategy (`0x5a8440`) and Wave_bindlocus_strategy (`0x5a8350`) override it; caller unknown |

Base-class defaults are one-instruction stubs `0x5aac00 0x5aac10 0x5aac40 0x5aac30 0x5aac60
0x5aac80 0x5aac70 0x5aac50` (slots 1..8; slot 5's returns 0).

**Rule** (**corrected**): each list equals the set of the part's slots whose class overrides that
method, in slot order, for lists `+0x28, +0x30, +0x34, +0x3c, +0x40, +0x44` in every part of every
file. The **draw list** equals the override set **or is empty**: 361 parts game-wide leave a drawing
class out (Anm_plus 124, Mdlts_plus 86, Megaanm_plus 61, Mdl_plus 59, ...), e.g. p_ac_com0 body 5
(REB1) parts 3 and 5. A player must use the file's lists. In the 19 Flowmotion elements of
section 11 the draw list equals the drawing classes in **171/171 parts** (39 parts draw nothing).

## 7. The interpreter

### 7.1 One frame

1. **Time task** `0x554f80` (priority 0xEA60, `0x557eb1`): dt = `0x6294e0() * 0.5`, dt2 =
   `0x540750() * 0.5` (`0x554fbd`, `0x554fd4`); both 0 when `0x60e160()` is true
   (`0x554fa0..0x554fac`). `0x5b55d0` gives each instance dt 0 (`+0x0a`), dt2 (`+0x0b`) or dt, 0
   during its start delay (`0x5b5689`), and calls `0x5b4290(ctx, inst, dt)`, which returns at once
   when dt == 0 (`0x5b42f4..0x5b4305`). Otherwise: a looping effect that is not stopping has dt > 10
   replaced by 1.0 (`0x5b4361`); effect time += dt (`0x5b436d..0x5b439d`); every part's time += dt
   (`0x5b4400`); a loopable part of a looping body past its loop end gets state 0x10; a part with
   lifetime <= int(time) (and no live children if flag bit 0) dies: release list, unlink
   (`0x5b4535..0x5b456a`, `0x5b82b0`); spawn events (section 5); body end (section 5).
2. **Pass-0 update** `0x554f70 -> 0x5b53e0` (priority 0x249F0, `0x557ec7`): unused by every file.
3. **Pass-1 update** from `0x557570` (reached from `0x5dffc0`): per layer, `0x5b47a0` (with view and
   distance tests, `0x5b48e9..0x5b4d31`) or `0x5b4f70` -> per instance `0x5ae260` (effect matrix from
   the attachment) then `0x5b4700`: phase 7 (first-update slots, `0x5b4755`), keys (`0x5b5040`,
   `0x5b476c`), phase 2 (update slots, `0x5b4788`); then `0x5b5e70`: phase 8 (`0x5577f3`).
4. **Draw** `0x5b59d0` / `0x5b5c20`: the draw list, in two passes split by the byte table
   `0x9e7c60` bit 0 (set only for id 50, FecLensflare_light); a command is skipped when
   `0x5e19b0() >= 0x50` (**GUESS** a draw-queue limit). What calls the draw wrappers
   (`0x5b59b0`/`0x5b5c00`, `0x5bc200`) was not traced.

**Uncertain**: that the time task runs before the updates (tasks are inserted in a list sorted by
ascending priority, `0x62a140 -> 0x629700 -> 0x629d30`, walk `0x629d60`; the runner was not read)
and that pass 1 runs after characters are posed (only `0x5e01ab` seen). **Unknown**: the unit of
`0x6294e0` (**GUESS**: 1.0 per 60 fps frame, so one FEP frame = 1/30 s).

`ctx+0x90` holds the phase: 0/1 (`0x5b55d0`), 2..5 (`0x5b53e0`), 6..9 (`0x5b4f70`/`0x5b4700`), 0xC
(`0x5b5e70`). When ctx bit 0 is set (**corrected**): slot 2 runs only for FecLmatrix (14) and
FecLmatrixbase (132) (`0x5b61b8`); slot 7 does not run (`0x5b5e7a`); slot 3 is not filtered; and
`0x5b55d0` passes dt = 0 (`0x5b5633`), so time stops.

**Element creation catch-up** (`0x5ac210`, `0x5ac74c..0x5ac826`): by `ctx+0x90` at creation, a new
element immediately gets: phase 0 -> time -= ctx dt; 1 -> a time step; 5 -> time step + its pass-0
update (`0x5b5350`); 9 -> time step + `0x5b5350` + `0x5ae260` + `0x5b4700`.

### 7.2 Calls into the command methods

Arguments (rcx = class object, rdx = ctx): slot 0 `(r8 params, r9 work)`, spawn info at `ctx+0x98`;
slot 1 `(r8 new params, r9 work, [5] old params, [6] f32 old span)`, where the old span is the
slot's previous entry `+6`, or the current frame when it had none (`0x5b5243`); slots 2, 3, 7
`(r8 params, xmm3 f32 local time, [5] work)`, local time = part time - current entry's start;
slot 4 `(r8 params, r9 work)`; slot 5 `(r8 params, xmm3 t_new, [5] work, [6] old params, [7] t_old)`.
Before each call the dispatchers set `ctx+0x78` = element instance, `+0x80` = part instance,
`+0x88` = command record. **Corrected**: the part-death release loop `0x5b82b0` does **not** set
them (stale values), skips slots whose params pointer `+0x68` is null and clears it after the call
(`0x5b8380`, `0x5b83ce`).

- **Init** (`0x5b56f0`): per init entry `+0x68[slot] = list + offset`, then slot 0. **Corrected**:
  if slot 0 of entry i fails, the clean-up `0x5b57e8..0x5b5863` calls **the failing slot's** release
  method (class and work of record i, `0x5b5816`) i+1 times with each earlier entry's params; the
  earlier slots' own releases never run.
- **Keys** (`0x5b5040` -> `0x5b5180`, skipped when dt == 0): if a wrap is pending, run the keys up to
  loop end - 1, rewind, set state 2, time = (f - end) mod (end - start) + start + frac. Then while
  the next key list's frame <= int(time): per entry, `old = +0x68[slot]`, `+0x68[slot] = new`,
  `+0x70[slot] = entry`, slot 1(new, work, old, old span). Entry order matters.
- **Loop rewind** (`0x5b6680`), **corrected**: `+0x46 = 0`; for each loop-list slot, `0x5b80c0`
  looks for the key active at loop start by **class id** (`0x5b676d`; compare `0x5b8198`), taking
  the first entry of a matching class in key lists with frame <= loop start, else falls back to the
  init list with t_new = 0 (`0x5b81df`, `0x5b67c5`); that entry becomes current and slot 5 runs;
  `+0x46` = found index + 1 maxed with the current value, or -1 when there is no later key or no
  match (`0x5b6814..0x5b684c`). Two slots of one class therefore get the same (first) key; this
  affects 13 looping Flowmotion parts with several Generateanchor slots (ask0 e0 p6/p18; com0 e10
  p0/p2/p3/p4, e12 p1, e13 p8, e14 p18/p22, e19 p2; bos_0 e0 p1, e1 p3). Exact replay must emulate it.
- **Spawn** (`0x5b73d0`): allocate (tag 0x75), header, zero `+0x68`/`+0x70`, init, link at the tail
  of its pass list, parent's child count +1, frame owner; during phases 8..9 (spawned by a
  generator in the pass-1 update) its keys up to int(time) run at once (`0x5b793c..0x5b7961`). A null
  part entry or a part with 0 slots aborts (`0x5b7428`, `0x5b7440`). `+0x46` = 0 if it has keys,
  else -1 (`0x5b7592..0x5b75a1`).

### 7.3 Runtime structures (decoder only unless noted)

- **Part instance** (`0x5b73d0`, checked `0x5b7512..0x5b76ae`): `+0x02` state (2 looped, 4 started,
  8 dead, 0x10 wrap pending, 0x40/0x80 from ref bit 13), `+0x04` part flags, `+0x08` pass,
  `+0x20` parent, `+0x28` frame owner, `+0x30` live children, `+0x40` f32 time, `+0x44` lifetime,
  `+0x46` next key, `+0x50` draw state (only with a draw list), `+0x58` part, `+0x60` -> work
  (`self+0xa0`), `+0x68` params per slot, `+0x70` current entry per slot, `+0x78` local / `+0x80`
  world / `+0x88` own frame matrix, `+0x90` matrix type, `+0x91` Lmatrix byte 1.
- There is **no fixed particle record**: each command keeps its state in its own slice of the work
  area, its output in the first vec4; consumers find it through their inputs. A "particle" is a
  child part spawned by a generator.
- **Spawn info** (`ctx+0x98`, read by slot 0; checked): `u32 flags`, then vec4 pointers at
  `+0x08 + 8*bit`: bits 0-2 position / velocity / acceleration, 3-5 rotation, 6-8 scale, 9-11
  colour. FecPosac **adds** them (`0x582e6f..0x582f69`); Pos, Posmv, Rot*, Scl*, Col* replace.
  Generate and Generateanchor pass flags 7 (`0x581c43..0x581c55`, `0x580405`); spawn events pass null.
- **Element instance** (0x1d0 bytes, `0x5bd400` in `0x5ac210`): `+0x04` body flags, `+0x10/+0x20/
  +0x30` position / rotation / scale, `+0x60` matrix (`0x5ae260`), `+0x138` time, `+0x13c` dt,
  `+0x144` kill timer, `+0x148` fade factor, `+0x168` start delay, `+0x16c` length, `+0x178/+0x180`
  pass lists, `+0x198` file, `+0x1a0` body, `+0x1a8` element, `+0x1b0` next spawn event.
- **Context** (one global, `0xaa3de0`): `+0x00` flags, `+0x10` dt, `+0x14` dt2, `+0x18` allocator,
  `+0x2c` category mask, `+0x78/+0x80/+0x88` current instance / part / record, `+0x90` phase,
  `+0x98` spawn info, `+0xa0` key lists run in this call, `+0xa8/+0xb0` loaded ANM / MDL tables.

## 8. Commands

The full per-class tables (RVAs of every slot, formulas) are in [commands.md](commands.md) section 6.
Summary with the skeptics' verdicts:

### 8.1 Pos / Rot / Scl / Col and their mv / ac forms (ids 1-9, 15-17): checked

Work: `+0x00` output vec4, `+0x10` base, `+0x20` velocity, `+0x30` acceleration. Params: value
(+ for Pos* a byte at `+0x0c`, 0 in all 745 blocks), velocity `+0x10`, acceleration `+0x20`.
Pos/Rot/Scl are vec3, Col vec4. Output: value form = base; mv = base + v t; ac = base + v t +
a t^2/2 (`0x582d80`, 0.5 at `0x867d7c`). **Units**: rotation in degrees (`0x565a3d`), scale in
percent (output x 0.01, `0x583dd0`, `[0x867d08]`), colour clamped to 0..255 (`0x57f1f0`,
`[0x78e78c]`). The files use 128 as "full" colour (**GUESS** 128 = 1.0, as on the PS2).

Key change of the ac forms (`0x5afe70`, vec4 `0x5b0090`; checked), t = old span:
`base' = pos_W(t) - pos_old(t) + new.p; vel' = vel_W(t) - vel_old(t) + new.v; acc' = acc - old.a +
new.a`: the deviation from the parameters (random offsets, spawn velocity, gravity) carries over.
Reproduced on data: p_fa_ajc0 e0 p2 Sclac 200 -> 341 -> 216.3, e0 p1 Colac 160 -> 128. Value and mv
forms and the loop-rewind formulas (`0x5b04e0`, `0x5b0720`) are in commands.md 6.1 (decoder only).
Pos/Posmv/Posac also override slot 6: a root part of matrix type 2 has its base position (y, z
negated) moved by the effect matrix once (`0x5b21a0`; decoder only).

### 8.2 FecLmatrix (14)

Inputs 0 = Pos*, 1 = Scl*, 2 = Rot* (`0x58260f..0x582694`). Slots 0 (`0x5825c0`) and 2 (`0x5821b0`):
local `+0x78` = S * R(order) * T after **negating y and z of position and rotation**
(`0x5826a1..0x5826d4`); param byte 3 = order 0..5 = ZYX, ZXY, YZX, YXZ, XZY, XYZ (jump table
`0x582afc`, RotZ `0x565b50`, RotY `0x55f3e0`, RotX `0x565a20`). Checked. **Not re-verified**:
the row-vector convention (`0x63ad40`) and the RotX/Y/Z matrix contents (commands.md 6.2).
Byte 0 = attach mode for `0x5b10e0`; it equals the part's matrix type in 706/706 Flowmotion blocks
(**not** a rule: 122 blocks elsewhere differ). Bytes 1 (-> instance `+0x91`; low 3 bits 1 =
camera-facing draw in FecMdl, **INFERRED**), 2 (orient along velocity, 0 in all files), 4, 5
(`0x5bc450`, **GUESS** sort group): decoder only. World matrix (`0x5b10e0`) and the part matrix
types (commands.md 7.2, 7.3): decoder only, partly followed.

### 8.3 Generators

- **FecGenerateanchor (151)**, checked: one burst per key list naming it (and per loop): slot 0
  `0x580340` pending = 0; slot 1 `0x5802d0` fires a pending burst **with the old block**, then
  pending = 1; slot 2 `0x580310` fires with the current block; slot 5 `0x580890` pending = 1.
  Spawning `0x580350`: `+0x00` count (truncated), `+0x04` random cube half-size, `+0x08` speed,
  `+0x0c` acceleration, `+0x10` child part (< 0 = off). Children start at time 0 with parent = this
  part and spawn flags 7 (`0x5b7980`).
- **FecGenerate (18)**, **uncertain** (partly re-verified): continuous emission, `+0x00` rate per
  frame, `+0x04` cube, `+0x08` speed, `+0x0c` acceleration (`0x581e54`, `0x581e73`), `+0x10` child
  (bit 12 = part group), `+0x12` interval, `+0x17` chance %. The accumulator logic `0x581a20` and
  that the direction is normalised (tail of `0x5b23b0`) were not re-verified.
- **FecGenerateanchorvertex (150)**: child ref at `+0x0c` (`0x580063`); reads a VTL index list
  through the file (`0x57fca5..0x57fcd5`). Otherwise **not decoded**.

### 8.4 FecGravity (19), checked

Slot 1 only (`0x585350`): input 0's (a Posac) acceleration.y (`work + 0x34`) += s16 g / 100 * 0.098
(`[0x867f14]`, `[0x7ed324]`). Slot 0 returns 1.

### 8.5 FecRndpos / Rndrot / Rndscl / Rndcol (22-25), checked except the tables

Slot 1 only (`0x5a9510`, `0x5a9ea0`, `0x5aa800`, `0x5a8a80`), skipped once the part has looped
(state bit 2, `0x5a9527`...). They perturb the input command's value / velocity / acceleration by a
random amount in [-r, r], [0, r] or [-r, 0]; enable and mode bytes per commands.md 6.6 (offset tables
`0x9e82fc..0x9e8334`: decoder only).

### 8.6 Drawing classes: resource index (checked) and what is not decoded

| class | draw slot | param `+0x0c` (s32) | other slots |
|---|---|---|---|
| FecMdl_plus (154) | `0x58a110` | section-3 (MDL) index: file `+0x34`, record handle `+0x10`, skipped if index < 0 or handle -1 (`0x58a135..0x58a18b`) | slot 0 `0x58a100` returns 1; slot 1 `0x58a0f0` is `ret` |
| FecMdlts_plus (155) | `0x588f90` | section-3 index (`0x588fbc..0x588fe4`, again `0x5892c6`) | slot 0 `0x588f60` copies param `+0x10..+0x27` (three vec2: value, velocity, acceleration) to work; slot 1 `0x588c80` rebases them like the ac forms and adds randoms from `+0x28..` if byte `+0x4a` and not looped; loop `0x589770` (decoder only; **GUESS** a texture scroll) |
| FecAnm_plus (152) | `0x586f10` | section-4 (ANM) index (`0x586f3e..0x586f55`) | slot 0 `0x586dc0` / slot 2 `0x586c80`: animation clock (time, frame, random start `+0x18`, speed `+0x10`, flip `+0x1a`; decoder only, commands.md 6.7) |
| FecMegaanm_plus (153) | `0x58e010` | section-4 index (`0x58e055..0x58e0a3`) | **its own code**, not FecMegaanm's (21): slot 0 `0x58df30` allocates `0x30 + 0x80*N` bytes, N = u16 param `+0xf0` (`0x58df56..0x58df70`), pointer in work `+0`, N at buffer `+0x1c`; buffer `+0x22` = 1 when the part's matrix type is 0, 3 or 4 and param byte `+0xfb` == 7. Slots 1 `0x58cc40`, 2 `0x58cdd0`, 4 `0x58dfe0`, 5 `0x58e7f0` not decoded |

In the 19 Flowmotion elements all 267 such indices name a non-empty record whose resource exists
with the right magic (`ddd_fep_fm_classes.py --draw`). **Not decoded at all**: the drawing itself
(every draw slot above), Bindlocus_polygon (105) + Wave_bindlocus_strategy (106), Shapetail (80) +
Locus_tail_strategy (68) + Null_routegenerator (79), Colorblink (62) + Colorblink_parameter (61),
Ellipse_line_particle (104), Radialg_line_particle (100). Their vtable RVAs are in section 11.2.

## 9. Resources (sections 2..5)

### 9.1 Records and registration (checked, 667)

24-byte records `{f32 1.0 (0.95 in 3 records game-wide; never read), u32 id, u32 size, u32 file
offset, i32 -1 (runtime: the manager slot, written at 0x5b30ef / 0x5b0b1f / 0x5b2c1f / 0x5b0fdf),
u32 0}`. A record with **id 0 is skipped** (`0x5b3076` and equivalents; 3173 game-wide).

| section | count / table | data magic | manager (system object field, global, vtable) | registered by |
|---|---|---|---|---|
| 2 | `+0x20` / `+0x30` | `TEX` | `+0xb8`, `0xaa3ca8`, CTextureManager `0x7eab18` (192 CEffTexture slots of 0x118 B) | `0x5b3020` |
| 3 | `+0x22` / `+0x34` | `MDL` | `+0xb0`, `0xaa3ca0`, CMdlManager `0x7ed718` | `0x5b2b50` |
| 4 | `+0x24` / `+0x38` | `ANM` | `+0xa8`, `0xaa3c98`, CAnm2DManager `0x7eabb8` | `0x5b0a50` |
| 5 | `+0x26` / `+0x3c` | `VTL` | `+0xc0`, `0xaa3cb0`, CFermdlManager `0x7eac58` | `0x5b0f10` |

The managers are built in `0x5552b0` and stored into the system object by `0x5b8780`, `0x5b8740`,
`0x5b8770`, `0x5b8750` (called `0x555689..0x5556c2`): a direct store, not an inference.
`(id & 0x78000000) == 0x70000000` -> **resident**: found by id among owner-0 slots (vtable `+0x28`:
TEX `0x55abb0`, ANM `0x55a450`, MDL `0x5bee80`; find `0x55ae10`); else **local**: vtable `+0x20`
(TEX `0x5c03a0`, ANM `0x55a4d0`, MDL `0x5bf540`, Fermdl `0x55abd0`) loads `FEP + offset` (rounded up
to 16, `0x55acac`; all local offsets are 16-aligned) and keys the slot by the id inside the data.
MDL and ANM loads resolve their texture lists with `find(owner, id)` on the texture manager, so
section 2 must be registered first (it is).

**Ids.** Local: `0x80000000 | (h>>28)<<27 | ((h>>24)&0xf)<<16 | ((h>>16)&0xff)<<8 | type<<7 |
index`, h = header `+0x10`, type 0 TEX / 1 ANM / 2 MDL / 3 VTL, index = position in the section
(5044/5044 local records, 667). Resident: `0xF0000000 | (k>>7)<<26 | type<<7 | (k&0x7f)` = entry k
of resident.frr's table `type` (326/326; every resident id used anywhere exists).

### 9.2 resident.frr

`FRR\0`, `+0x0c` 4 tables, `+0x10` size - 1, `+0x14` counts (32 TEX, 134 ANM, 136 MDL, 24 VTL),
`+0x24` four tables of -1 (runtime slots, **INFERRED**), `+0x34` four tables of resource offsets
(TEX, ANM, MDL, VTL order). Each resource is the same block as in an FEP. The code that registers
them with owner 0 was not found. `+0x04` (0x80003800) and `+0x08` (1) unknown.

### 9.3 TEX / CTRT textures (checked)

TEX: `+0x1c/+0x20` width/height, `+0x24` CTRT offset (0x80); CTRT `+0x08` info offset (0x10),
`+0x0c` pixel offset (0x80); info `+0x0c` PICA200 format 0..13 (larger -> 0, `0x5fd062`),
`+0x10/+0x12` u16 decode size (the size used, `0x5fd03f`, `0x671493`), `+0x14` mip count (0).
Path: `0x5c03f0 -> 0x5fd030 -> 0x5fc2f0` (replacement lookup) `-> 0x671480 -> 0x6714aa` (HD
replacement, below) -> decoder by jump table `0x67193c` -> RGBA8 buffer (byte 0 R, row 0 = first data
row, no flip, straight alpha) -> `0x677e70`. 8x8 tiles row-major from the top, Morton order inside
(x in bit 0; table at `.rdata 0x81b4e0`). Formats and ETC1 details: textures.md section 3 (all
decoders re-read). **ETC1/ETC1A4 base colours are not bit-replicated** (`c4<<4`, `c5<<3`,
`0x671c67..0x671c9f`, `0x66eef0`): the 1x remastered DDS files equal this decode bit for bit, while a
standard ETC1 decoder differs by up to 14 levels. `ddd_fep_textures.py` reproduces the exe exactly:
16 of the 32 resident textures have a 1x remastered DDS, and all 16 are bit-identical to its output.
Resident `0xF000000F` (used by p_ac_com0) decodes as 128x80 from 0x4000 bytes: rows 64..79 are the
next resource's bytes, and the remastered `-15.dds` reproduces them.

**HD replacement** (**corrected**: textures.md says there is none). The zero-name check in
`0x5fc2f0` (memcmp import `0x771bf8`) is real, but `0x671480` then calls `0x678430` (`0x6714aa`),
which matches the texture by a per-file ordinal (`0x679e90` / `0x67a1a0`, name `'%s%d'` `0x7869bc`)
against the list `[rec+0x670]` filled by `0x637f30` from `-<n>.dds/.png` assets, and on a match
creates the texture from the replacement (`0x678130`) and skips decoding. Present:
`remastered/effect/resident/bin/resident.frr/-0..-31.dds` (index k = resident TEX k; sizes 1x/2x/4x)
and `remastered/effect/sys/ffa/bin/p_fa_ajc0.fep/-0.dds` (2x of `0x9803AC06`). **For Flowmotion, 9 of
the 18 textures are replaced by different art**: F0000001 (4x, different noise and alpha),
F0000005, F0000007, F000000C, F0000012, F0000015 (512x512, same layout, different alpha), F0000016,
F000001B and 0x9803AC06 (2x). The other nine (F0000004/06/08/09/0F/18/1C/1D/1E) are 1x and identical
to the decode. To look like the PC game, a port uses the remastered DDS (same UV layout).
**INFERRED**: the runtime ordinal equals the frr / section-2 order, and the mode global `0x14e61000`
is 0 in play.

Flowmotion texture inventory (52 slots, 18 distinct, only `0x9803AC06` stored locally): textures.md
section 4.

### 9.4 MDL models

Header: `MDL\0`, u16 1, u16 10, `+0x08` id, `+0x0c` loaded flag, **`+0x10` texture-reference count,
`+0x14` its offset (0x20)** (**corrected**, from CMdlManager's load `0x5bf590`): 24-byte entries
`{u32 texture id, i32 slot (written 0x5bf644/0x5bf650; -1 for id 0 at 0x5bf66b), u32 flags
(7/0xF/0x2B/0x2F), u32 blend code, u32, i32}`; the meshes follow (mesh offset = 0x20 + 24n in all 91
Flowmotion MDLs). Every texture id an MDL or ANM names is in its FEP's section 2 (667).
`+0x18` mesh count, `+0x1c` mesh offset. Tree from CMdl::Load `0x5bf7a0` (checked): mesh 0x44 bytes
(offsets `+0x04 +0x0c +0x14 +0x18 +0x20` relocated, `+0x1c` sub count); sub 16 bytes (`+0x08` group
count, `+0x0c` group offset); group 16 bytes (`+0x04`, `+0x0c` relocated; when mesh `+0x10` > 1,
`+0x04` points to `[group+8]` u32 offsets that are relocated too, `0x5bf8e3..0x5bf915`). Offsets are
from the MDL start in the file.

**Vertex format (new, INFERRED from the data):** sub `+0x00` is a **PSP GE vertex type** and group
`+0x00` a **PSP GE primitive type**; group `+0x08` = vertex count, `+0x0c` = vertex offset.
Vertex types seen: `0x11e` (u16 u, v; u32 colour 8888; s16 x, y, z; padded to 16 bytes), `0x11c`
(colour + position, 12 bytes), `0x102` (u16 u, v; s16 x, y, z; 10 bytes), `0x100` (position only,
6 bytes). With those sizes the vertex blocks of consecutive groups follow each other (16-aligned) at
**1590 of 1591 boundaries over all 155 models involved** (every local MDL of the 7 Flowmotion FEPs
and all 136 resident ones; the one exception is a 32-byte gap in resident model 51). Primitive 4
(triangle strip) in 1676 groups, 3 (triangles) in 70. Mesh `+0x10` is > 1 exactly for the vertex
types with colour (0x11e, 0x11c). The topic spec's "16-byte vertex" holds only for 0x11e; it gives
garbage for the 10-byte 0x102 models (e.g. 0x9803AD0A, 0xF0000118, 0xF000013D).
**INFERRED scale: 16384 = 1.0** for both UV and position. Positions span -16384..16384 in nearly
every model, and the smallest UVs sit one texel inside the edge (u 129 on 128-wide textures, v 260
on 64-high ones). The exe's reader of sub `+0` / group `+0` was not traced. Still unknown: the
mesh f32 fields (`+0x24..+0x2c`, `+0x38..+0x40`), the colour tables at mesh `+0x0c`/`+0x14`, the
group `+0x04` per-vertex u32 tables, the texture-ref flags and the last two fields.

### 9.5 ANM 2D animations

CAnm2D::Load `0x55a5f0` (checked): `+0x14` total length (= sum of key durations, 21/21 Flowmotion
ANMs), `+0x18` texture count / `+0x28` offset (8-byte entries `{u32 texture id, i32 runtime slot
written at 0x55a5b0}`), `+0x1c` keys / `+0x2c` (12 bytes `{u32 flags: 0 or 0x80, u32 frame, u32
duration}`), `+0x20` frames / `+0x30` (12 bytes `{u32 8/0x88/0x80, u32 n, u32 -> n quad indices}`,
`+8` relocated `0x55a633`), `+0x24` quads / `+0x34` (0x1c bytes `{0, ?, u32 4 vertices, u32 ->
vertices (relocated 0x55a661), u32 blend code, 0, -1}`; blend code read at `0x55a13b..0x55a155` ->
`0x5c02a0`: 0x70600 -> 1, 0xA0600 -> 2, 0xA0602 -> 3, else 0). All 394 Flowmotion quads use 0xA0600.
**Quad vertices (new, INFERRED)**: 10 bytes `{u16 u, u16 v, s16 x, s16 y, s16 z = 0}`, the layout of
vertex type 0x102, with **u, v / 16384** in texture space. In the 73 resident ANMs that use the
atlas F0000015, all 5580 vertices have u and v at multiples of 64 (whole texels of the 256-pixel
atlas) and z = 0. Most sit one texel inside 16/32-pixel cells (e.g. a cell u 2..30, v 1..31 px).
textures.md tried 1/4096 and 1/8192 and failed. Vertex order in the examples: the two top corners,
then the two bottom ones (a strip).

### 9.6 VTL (section 5)

CFermdl's Load is `ret 0` (`0x5c0af0`, vtable `0x7eac20 [4]`); section 5 = CFermdlManager (direct
store `0x5b8750`). Layout checked on all 208 VTLs: `VTL\0`, id, u16 1, u16 2, 0, 0, u16 0, u16 list
count `+0x16`, `+0x18` = 0x1c -> `{u32 n, u32 offset}` -> n u16. Used by FecGenerateanchorvertex
(section 8.3; **INFERRED** vertex orderings).

## 10. ESE (sound list)

Matched to the FEP **by name**, not by position: on first use `0x559cf0` looks up each entry's name
with `0x557bd0 -> 0x5abb70`, stores the section-7 index at entry `+2` and sets ESE `+8` bit 0
(`0x559d31..0x559d64`). 675 of 2097 entries game-wide would be wrong by position; 34 stored indices
are stale; 10 entries name elements missing from their FEP (index -1). Layout: `"ESE\0"`, `+0x08`
runtime flags, `+0x0c` u16 2, `+0x0e` entry count, `+0x10` s32[] entry offsets; entry `{u16 sound
count (>= 1 always: only elements with a sound appear), s16 name index, char[16] name, n x
{char[32] sound, u32}}`, entries 16-aligned with 0xCD padding. The u32 after a sound is 30 (10 for
ASK MLC0; **GUESS** u16 + u16). p_fa_bos_0 and w_010_so have no ESE.

## 11. Flowmotion: the command classes each element uses

Produced by `tools/ddd_fep_fm_classes.py` (it reuses `ddd_fep_dump.py`'s parser) on the seven FEPs.
"x n" = number of commands (slots) of that class over the element's parts. "loops" = body flag bit 1.
"stop" = body `+0x0c` (0 = fade 30 frames on a stop request, 3 = ignore it). The elements are
those of `ddd-notes.md`: ajc0 = the dive, akd0 = kick and slam, ask0 = pole attack, com0 = wall run,
rail, trails, throws, p_ac_com0 = dash / glide / ring, bos_0 = rail, w_010_so LOC0 = Keyblade trail.

### 11.1 Per element

| element | file : body | frames | loops | stop | parts | command classes (id name x commands) |
|---|---|---|---|---|---|---|
| P_FA_AJC_CAL0_0 | p_fa_ajc0 : 0 | 35 | no | 3 | 6 | 1 Pos x2, 2 Posmv x1, 3 Posac x3, 4 Rot x5, 6 Rotac x1, 7 Scl x4, 9 Sclac x2, 14 Lmatrix x6, 15 Col x2, 17 Colac x3, 105 Bindlocus_polygon x1, 106 Wave_bindlocus_strategy x1, 150 Generateanchorvertex x2, 152 Anm_plus x1, 153 Megaanm_plus x2, 154 Mdl_plus x1 |
| P_FA_AJC_SHO0_0 | p_fa_ajc0 : 1 | 45 | no | 0 | 20 | 1 Pos x8, 3 Posac x12, 4 Rot x18, 5 Rotmv x1, 6 Rotac x1, 7 Scl x6, 9 Sclac x14, 14 Lmatrix x20, 15 Col x2, 17 Colac x16, 23 Rndrot x1, 68 Locus_tail_strategy x1, 79 Null_routegenerator x1, 80 Shapetail x1, 150 Generateanchorvertex x1, 151 Generateanchor x9, 152 Anm_plus x4, 153 Megaanm_plus x2, 154 Mdl_plus x1, 155 Mdlts_plus x10 |
| P_FA_AKD_DRL0_0 | p_fa_akd0 : 0 | 30 | yes | 3 | 11 | 1 Pos x8, 3 Posac x3, 4 Rot x5, 5 Rotmv x2, 6 Rotac x2, 7 Scl x5, 9 Sclac x4, 14 Lmatrix x9, 15 Col x1, 17 Colac x8, 18 Generate x2, 22 Rndpos x1, 23 Rndrot x2, 24 Rndscl x2, 152 Anm_plus x2, 153 Megaanm_plus x1, 154 Mdl_plus x3, 155 Mdlts_plus x3 |
| P_FA_AKD_CAL0_0 | p_fa_akd0 : 1 | 30 | no | 3 | 5 | 1 Pos x3, 3 Posac x2, 4 Rot x4, 6 Rotac x1, 7 Scl x2, 9 Sclac x3, 14 Lmatrix x5, 15 Col x1, 17 Colac x3, 105 Bindlocus_polygon x1, 106 Wave_bindlocus_strategy x1, 150 Generateanchorvertex x2, 152 Anm_plus x1, 153 Megaanm_plus x1, 154 Mdl_plus x1 |
| P_FA_AKD_SHO0_0 | p_fa_akd0 : 2 | 45 | no | 3 | 16 | 1 Pos x12, 3 Posac x4, 4 Rot x10, 5 Rotmv x1, 6 Rotac x5, 7 Scl x3, 9 Sclac x13, 14 Lmatrix x16, 15 Col x2, 17 Colac x14, 23 Rndrot x1, 152 Anm_plus x4, 153 Megaanm_plus x2, 154 Mdl_plus x2, 155 Mdlts_plus x8 |
| P_FA_ASK_MLC0_0 | p_fa_ask0 : 0 | 30 | yes | 3 | 19 | 1 Pos x3, 3 Posac x16, 4 Rot x10, 6 Rotac x9, 7 Scl x14, 9 Sclac x5, 14 Lmatrix x19, 15 Col x1, 17 Colac x13, 23 Rndrot x2, 61 Colorblink_parameter x1, 62 Colorblink x1, 104 Ellipse_line_particle x1, 151 Generateanchor x16, 152 Anm_plus x2, 153 Megaanm_plus x1, 154 Mdl_plus x2, 155 Mdlts_plus x8 |
| P_FA_COM_WAL0_0 | p_fa_com0 : 11 | 20 | no | 0 | 14 | 1 Pos x1, 3 Posac x13, 4 Rot x9, 5 Rotmv x3, 6 Rotac x2, 7 Scl x7, 9 Sclac x7, 14 Lmatrix x14, 15 Col x1, 17 Colac x10, 22 Rndpos x3, 23 Rndrot x7, 25 Rndcol x2, 151 Generateanchor x15, 152 Anm_plus x4, 153 Megaanm_plus x1, 154 Mdl_plus x5, 155 Mdlts_plus x1 |
| P_FA_COM_BSS0_0 | p_fa_com0 : 12 | 30 | yes | 3 | 13 | 1 Pos x9, 3 Posac x4, 4 Rot x13, 7 Scl x10, 9 Sclac x3, 14 Lmatrix x13, 15 Col x2, 17 Colac x6, 18 Generate x3, 23 Rndrot x1, 61 Colorblink_parameter x4, 62 Colorblink x4, 100 Radialg_line_particle x1, 151 Generateanchor x5, 152 Anm_plus x2, 153 Megaanm_plus x1, 154 Mdl_plus x1, 155 Mdlts_plus x3 |
| P_FA_COM_LOC0_0 | p_fa_com0 : 9 | 40 | yes | 3 | 2 | 3 Posac x2, 4 Rot x2, 7 Scl x2, 14 Lmatrix x2, 17 Colac x2, 68 Locus_tail_strategy x2, 79 Null_routegenerator x2, 80 Shapetail x2 |
| P_FA_COM_LOC1_0 | p_fa_com0 : 10 | 40 | yes | 3 | 7 | 1 Pos x2, 3 Posac x5, 4 Rot x2, 5 Rotmv x5, 7 Scl x6, 9 Sclac x1, 14 Lmatrix x7, 17 Colac x4, 18 Generate x1, 23 Rndrot x1, 105 Bindlocus_polygon x2, 106 Wave_bindlocus_strategy x2, 151 Generateanchor x8, 152 Anm_plus x2 |
| P_FA_COM_LOC2_0 | p_fa_com0 : 19 | 40 | yes | 3 | 3 | 1 Pos x1, 3 Posac x2, 5 Rotmv x3, 7 Scl x3, 14 Lmatrix x3, 17 Colac x1, 23 Rndrot x1, 105 Bindlocus_polygon x1, 106 Wave_bindlocus_strategy x1, 151 Generateanchor x4 |
| P_FA_COM_MLC0_0 | p_fa_com0 : 16 | 20 | no | 3 | 11 | 3 Posac x11, 4 Rot x9, 6 Rotac x2, 7 Scl x4, 9 Sclac x7, 14 Lmatrix x11, 17 Colac x9, 23 Rndrot x1, 151 Generateanchor x12, 152 Anm_plus x2, 154 Mdl_plus x3, 155 Mdlts_plus x4 |
| P_FA_COM_FIR0_0 | p_fa_com0 : 17 | 20 | no | 3 | 18 | 1 Pos x6, 3 Posac x12, 4 Rot x14, 5 Rotmv x2, 6 Rotac x2, 7 Scl x7, 9 Sclac x11, 14 Lmatrix x18, 15 Col x2, 17 Colac x11, 23 Rndrot x2, 151 Generateanchor x18, 152 Anm_plus x4, 153 Megaanm_plus x2, 154 Mdl_plus x4, 155 Mdlts_plus x3 |
| P_AC_COM_MSL0_0 | p_ac_com0 : 0 | 30 | yes | 3 | 2 | 1 Pos x2, 4 Rot x2, 7 Scl x2, 14 Lmatrix x2, 17 Colac x2, 23 Rndrot x2, 155 Mdlts_plus x2 |
| P_AC_COM_MSS0_0 | p_ac_com0 : 1 | 30 | yes | 3 | 7 | 1 Pos x6, 3 Posac x1, 4 Rot x6, 5 Rotmv x1, 7 Scl x6, 9 Sclac x1, 14 Lmatrix x7, 17 Colac x5, 23 Rndrot x2, 151 Generateanchor x2, 154 Mdl_plus x1, 155 Mdlts_plus x4 |
| P_AC_COM_RNG0_0 | p_ac_com0 : 6 | 30 | no | 0 | 5 | 1 Pos x1, 3 Posac x4, 4 Rot x5, 7 Scl x1, 9 Sclac x4, 14 Lmatrix x5, 17 Colac x4, 23 Rndrot x2, 151 Generateanchor x4, 152 Anm_plus x1, 154 Mdl_plus x2, 155 Mdlts_plus x1 |
| P_FA_BOS_BSS0_0 | p_fa_bos_0 : 0 | 30 | yes | 3 | 6 | 1 Pos x4, 3 Posac x2, 4 Rot x6, 7 Scl x4, 9 Sclac x2, 14 Lmatrix x6, 15 Col x1, 17 Colac x3, 61 Colorblink_parameter x2, 62 Colorblink x2, 151 Generateanchor x5, 152 Anm_plus x1, 153 Megaanm_plus x1, 154 Mdl_plus x1, 155 Mdlts_plus x1 |
| P_FA_BOS_LOC1_0 | p_fa_bos_0 : 1 | 40 | yes | 3 | 5 | 1 Pos x1, 3 Posac x4, 4 Rot x1, 5 Rotmv x4, 7 Scl x4, 9 Sclac x1, 14 Lmatrix x5, 17 Colac x2, 23 Rndrot x1, 105 Bindlocus_polygon x1, 106 Wave_bindlocus_strategy x1, 151 Generateanchor x5, 152 Anm_plus x1 |
| W_010_SO_LOC0_0 | w_010_so : 5 | 6 | yes | 3 | 1 | 1 Pos x1, 4 Rot x1, 7 Scl x1, 14 Lmatrix x1, 17 Colac x1, 105 Bindlocus_polygon x1, 106 Wave_bindlocus_strategy x1 |

All 19 elements: 171 parts, every part in pass 1, every part's draw list = its drawing classes.

### 11.2 The union: what the mod must implement (31 classes)

"commands" = slots over the 19 elements, "parts" = parts holding the class, "elems" = elements using
it. "phases" = the slot lists it sits in (update = slot 2, draw = 3, release = 4, loop = 5, first = 6,
m40 = 8; a class in no list acts only through init and key). "own slots" = the vtable slots that are
not base defaults (slot: RVA).

| id | class | commands | parts | elems | phases | own slots | decoded? |
|---|---|---|---|---|---|---|---|
| 14 | FecLmatrix | 169 | 169 | 19 | first, update | 0 `0x5825c0`, 1 `0x5821a0`, 2 `0x5821b0`, 6 `0x582d30` | yes (8.2); multiply convention, bytes 1/2/4/5, `0x5b10e0` modes partly |
| 4 | FecRot | 122 | 122 | 18 | loop | 0 `0x583d00`, 1 `0x583ca0`, 5 `0x583d50` | yes |
| 17 | FecColac | 117 | 117 | 19 | loop, update | 0 `0x57f2f0`, 1 `0x57f1d0`, 2 `0x57f1f0`, 5 `0x57f3e0` | yes |
| 151 | FecGenerateanchor | 103 | 36 | 12 | loop, update | 0 `0x580340`, 1 `0x5802d0`, 2 `0x580310`, 5 `0x580890` | yes (8.3); child placement `0x5b23b0` tail not read |
| 3 | FecPosac | 100 | 100 | 17 | first, loop, update | 0 `0x582e10`, 1 `0x582d60`, 2 `0x582d80`, 5 `0x582f90`, 6 `0x583080` | yes |
| 7 | FecScl | 91 | 91 | 19 | loop | 0 `0x584320`, 1 `0x5842b0`, 5 `0x5843a0` | yes |
| 9 | FecSclac | 78 | 78 | 15 | loop, update | 0 `0x583e60`, 1 `0x583d70`, 2 `0x583dd0`, 5 `0x583f30` | yes |
| 1 | FecPos | 70 | 70 | 17 | first, loop | 0 `0x583570`, 1 `0x583510`, 5 `0x5835c0`, 6 `0x5835e0` | yes |
| 155 | FecMdlts_plus | 48 | 48 | 12 | draw, loop | 0 `0x588f60`, 1 `0x588c80`, 3 `0x588f90`, 5 `0x589770` | model index only; draw and the vec2 work not decoded |
| 152 | FecAnm_plus | 31 | 31 | 14 | draw, update | 0 `0x586dc0`, 1 `0x586c70`, 2 `0x586c80`, 3 `0x586f10` | clock (decoder only) + index; draw not decoded |
| 154 | FecMdl_plus | 27 | 27 | 13 | draw | 0 `0x58a100`, 1 `0x58a0f0`, 3 `0x58a110` | model index only; draw not decoded |
| 23 | FecRndrot | 26 | 26 | 14 | (key only) | 0 `0x5aa220`, 1 `0x5a9ea0` | yes (mode tables decoder only) |
| 6 | FecRotac | 25 | 25 | 9 | loop, update | 0 `0x583760`, 1 `0x5836b0`, 2 `0x5836e0`, 5 `0x583800` | yes |
| 5 | FecRotmv | 22 | 22 | 9 | loop, update | 0 `0x583a80`, 1 `0x583950`, 2 `0x583a40`, 5 `0x583af0` | yes (decoder only) |
| 15 | FecCol | 15 | 15 | 10 | loop | 0 `0x57f950`, 1 `0x57f880`, 5 `0x57fa00` | yes |
| 153 | FecMegaanm_plus | 14 | 14 | 10 | draw, loop, release, update | 0 `0x58df30`, 1 `0x58cc40`, 2 `0x58cdd0`, 3 `0x58e010`, 4 `0x58dfe0`, 5 `0x58e7f0` | allocation + ANM index only |
| 105 | FecBindlocus_polygon | 7 | 7 | 6 | draw, loop, release, update | 0 `0x59dcf0`, 1 `0x59d2a0`, 2 `0x59d360`, 3 `0x59deb0`, 4 `0x59de50`, 5 `0x59e5d0` | no |
| 106 | FecWave_bindlocus_strategy | 7 | 7 | 6 | loop, m40, release, update | 0 `0x5a82c0`, 1 `0x5a82a0`, 2 `0x5a82b0`, 4 `0x5a8320`, 5 `0x5a8370`, 8 `0x5a8350` | no (slot 8 caller unknown) |
| 61 | FecColorblink_parameter | 7 | 7 | 3 | loop, release, update | 0 `0x5852c0`, 1 `0x584ff0`, 2 `0x585050`, 4 `0x585330`, 5 `0x585340` | no |
| 62 | FecColorblink | 7 | 7 | 3 | loop, release, update | 0 `0x584c70`, 1 `0x584af0`, 2 `0x584b40`, 4 `0x584cb0`, 5 `0x584cc0` | no |
| 18 | FecGenerate | 6 | 6 | 3 | loop, update | 0 `0x582110`, 1 `0x5819f0`, 2 `0x581a20`, 5 `0x582120` | partly (8.3) |
| 150 | FecGenerateanchorvertex | 5 | 3 | 3 | loop, update | 0 `0x57fb60`, 1 `0x57faf0`, 2 `0x57fb30`, 5 `0x5802c0` | no |
| 22 | FecRndpos | 4 | 4 | 2 | (key only) | 0 `0x5a98a0`, 1 `0x5a9510` | yes (tables decoder only) |
| 68 | FecLocus_tail_strategy | 3 | 3 | 2 | loop, release, update | 0 `0x5a2110`, 1 `0x5a20f0`, 2 `0x5a2100`, 4 `0x5a21b0`, 5 `0x5a21d0` | no |
| 79 | FecNull_routegenerator | 3 | 3 | 2 | release, update | 0 `0x5853d0`, 2 `0x5853c0`, 4 `0x5853f0` | no |
| 80 | FecShapetail | 3 | 3 | 2 | draw, loop, release, update | 0 `0x5a24c0`, 1 `0x5a2200`, 2 `0x5a2390`, 3 `0x5a2850`, 4 `0x5a27d0`, 5 `0x5a34a0` | no |
| 24 | FecRndscl | 2 | 2 | 1 | (key only) | 0 `0x5aabf0`, 1 `0x5aa800` | yes (tables decoder only) |
| 25 | FecRndcol | 2 | 2 | 1 | (key only) | 0 `0x5a8f10`, 1 `0x5a8a80` | yes (tables decoder only) |
| 2 | FecPosmv | 1 | 1 | 1 | first, loop, update | 0 `0x583290`, 1 `0x583150`, 2 `0x583240`, 5 `0x583300`, 6 `0x583440` | yes (decoder only) |
| 104 | FecEllipse_line_particle | 1 | 1 | 1 | draw, loop, release, update | 0 `0x560070`, 1 `0x55fa10`, 2 `0x55fa20`, 3 `0x560130`, 4 `0x560100`, 5 `0x560980` | no |
| 100 | FecRadialg_line_particle | 1 | 1 | 1 | draw, loop, release, update | 0 `0x576610`, 1 `0x5765a0`, 2 `0x5765b0`, 3 `0x576730`, 4 `0x576710`, 5 `0x576770` | no |

Not needed for the 19 elements but used elsewhere in the same files: FecGravity (w_010_so HIT0/STR0),
FecFlash (p_fa_com0 FSH0), FecMask (p_ac_com0 WRP1). Never used by these files: Sclmv, Colmv, Anm,
Megaanm, Mdl, Mdlts (only their `_plus` forms), Rndpos1.

Which elements use which class (from the same run):

- Generate: AKD DRL0, COM BSS0, COM LOC1. Generateanchorvertex: AJC CAL0, AJC SHO0, AKD CAL0.
- Generateanchor: AJC SHO0, ASK MLC0, COM WAL0/BSS0/LOC1/LOC2/MLC0/FIR0, AC MSS0/RNG0, BOS BSS0/LOC1.
- Bindlocus_polygon + Wave_bindlocus_strategy: AJC CAL0, AKD CAL0, COM LOC1, COM LOC2, BOS LOC1,
  W_010 LOC0.
- Shapetail + Locus_tail_strategy + Null_routegenerator: AJC SHO0, COM LOC0.
- Colorblink + Colorblink_parameter: ASK MLC0, COM BSS0, BOS BSS0. Ellipse_line_particle: ASK MLC0.
  Radialg_line_particle: COM BSS0.
- Rndpos: AKD DRL0, COM WAL0. Rndscl: AKD DRL0. Rndcol: COM WAL0. Posmv: AJC CAL0.
- Megaanm_plus: AJC CAL0/SHO0, AKD DRL0/CAL0/SHO0, ASK MLC0, COM WAL0/BSS0/FIR0, BOS BSS0.
- Mdlts_plus: every element except AJC CAL0, AKD CAL0, COM LOC0/LOC1/LOC2, BOS LOC1, W_010 LOC0.

## 12. The dive (p_fa_ajc0) and the slam (p_fa_akd0), part by part

From `ddd_fep_fm_classes.py --parts --draw` (draw resource), the spawn events / generator
parameters in the dumps (start), and `ddd_fep_mdlshape.py` (shape: **INFERRED** from the vertex
format of 9.4, radii relative to the model's largest, 16384 = 1.0). Every model and animation below
uses blend code 0xA0600 and MDL texture flags 7.

**P_FA_AJC_SHO0_0** (body 1, 45 frames, not looping, stop mode 0):

| part | life | mtx | starts | draws | resource -> texture | shape |
|---|---|---|---|---|---|---|
| 0 | 20 | 1 | frame 0 | Anm_plus | ANM F00000AB -> F0000015 atlas | 1 quad |
| 1, 2 | 15 | 3 | frame 0 | Anm_plus | ANM F00000A3 / F0000087 -> F0000015 | 1 quad |
| 3 | 8 | 3 | frame 0 | Anm_plus (+Rndrot) | ANM F00000B6 -> F0000008 | 1 quad |
| 4 | 7 | 1 | frame 0 | Mdlts_plus | MDL F0000158 -> F0000007 | pointed shape on a square base (x, z within +-0.67), height 1 |
| 5 | 15 | 0 | frame 0 | Mdlts_plus | MDL F000013D -> F0000015 | vertical quad (z = 0), vtype 0x102 |
| 6 | 16 | 0 | frame 0 | Mdlts_plus | MDL F0000140 -> F0000001 | vertical strip (z = 0), width 0.38, height 1 |
| 7, 8 | 14, 15 | 0 | frame 0 | Mdlts_plus | MDL F0000145 -> F0000015 | thin cylinder, radius 0.19, height 1 |
| 9 | 15 | 0 | frame 0 | Mdlts_plus | MDL F000011A -> F0000001 | cylinder wall, radius 0.55..0.98, height 1 |
| 10 | 13 | 1 | frame 0 | Mdl_plus | MDL 9803AD0A -> F0000004 | vertical shape (z = 0), vtype 0x102 |
| 11 | 25 | 1 | frame 6 | Megaanm_plus | ANM F00000A7 (12 quads) -> F0000015 | sprites |
| **12** | 16 | 3 | child of part 16 | Mdlts_plus | MDL 9803AD0B -> F0000001 | **flat ring** (y = 0), radii 0.37 / 0.70 / 1.0, colour 0 inside -> 0x80 outside |
| **13** | 22 | 3 | child of part 16 | Mdlts_plus | MDL 9803AD04 -> F0000018 | **flat thin ring** (y = 0), radii 0.73..1.0 |
| **14** | 21 | 3 | child of part 16 | Mdlts_plus | MDL 9803AD0E -> F000001B | **low dome ring**, radii 0.44..1.0, height 0..0.18 |
| **15** | 21 | 3 | child of part 16 | Mdlts_plus (its parameter blocks name MDL 9803AD0D and 9803AD0C) | -> 9803AC06 (the only local texture) | same low dome ring |
| 16 | 29 | 1 | frame 2 | - (9 Generateanchor) | children 12, 13, 14, 15 (one block off) | emitter |
| 17 | 31 | 0 | frame 12 | Megaanm_plus | ANM 9803AC81 (32 quads) -> F0000015 | sprites |
| 18 | 19 | 0 | child of part 19 | Shapetail (+Locus_tail, Null_route) | - | trail |
| 19 | 20 | 1 | frame 0 | - (Generateanchorvertex) | child 18 | emitter |

**P_FA_AKD_SHO0_0** (body 2, 45 frames, not looping, stop mode 3):

| part | life | mtx | starts | draws | resource -> texture | shape |
|---|---|---|---|---|---|---|
| 0, 1, 6 | 17, 17, 19 | 1 | frame 0 | Anm_plus | ANM F0000087 / F00000AB / F00000AB -> F0000015 | 1 quad |
| 4 | 17 | 1 | frame 0 | Anm_plus | ANM F0000080 -> F0000012 | 1 quad |
| **2** | 12 | 1 | frame 0 | Mdlts_plus | MDL 9803990C -> F0000001 | **flat ring** (y = 0), radii 0.39..1.0 |
| 3 | 17 | 1 | frame 0 | Mdlts_plus | MDL F0000102 -> F000001C | vertical disc (z = 0) |
| 5 | 15 | 1 | frame 0 | Mdl_plus | MDL 9803990E -> F0000012 | vertical shape (z = 0) |
| **7, 8** | 16, 17 | 1 | frames 1, 3 | Mdlts_plus | MDL 98039903 -> F0000001 | **cup / dome**, radii 0.27..1.0, height 0.03..1 |
| **9** | 14 | 1 | frame 2 | Mdlts_plus | MDL 9803990B -> F0000001 | **ring rising** to height 0.5, radii 0.5..1.0 |
| **10** | 14 | 1 | frame 5 | Mdlts_plus | MDL 9803990D -> F0000001 | **band** (height +-0.16), radii 0.8..1.0, about 3/4 of a circle |
| **11** | 17 | 4 | frame 5 | Mdlts_plus | MDL F000017C -> F0000001 | **cylinder**, radius 0.49, height 1 |
| 12 | 15 | 3 | frame 1 | Mdl_plus | MDL 9803990A -> F0000004 | ring-like shell, height 0.05..0.6, vtype 0x102 |
| 13 | 12 | 1 | frame 7 | Mdlts_plus (+Rndrot) | MDL 98039902 -> F000001D | irregular |
| 14, 15 | 37, 30 | 0, 1 | frames 2, 11 | Megaanm_plus | ANM 98039881 (32 quads) / 98039885 (18) -> F0000015 | sprites |

**INFERRED**: the shockwaves are the flat rings, domes and cylinders in bold: Mdlts_plus models
textured with F0000001 noise (replaced in the PC game by a 4x DDS, 9.3), F0000018, F000001B and
0x9803AC06, sized by their Scl/Sclac (percent) and faded by Colac. So drawing them needs, in
order: the Pos/Rot/Scl/Col families, Lmatrix, Generateanchor (dive rings are children of part 16),
the MDL vertex data (9.4) and the Mdlts_plus draw (not decoded).

## 13. Unknowns, in priority order (dive and slam shockwaves first)

1. **How Mdlts_plus and Mdl_plus draw a model** (`0x588f90`, `0x58a110`). Open points:
   - which matrix is used (local `+0x78`, world `+0x80`, or the frame applied at draw time: INFERRED,
     commands.md 7.2);
   - how the Colac output (0..255), the vertex colour (max 0x80808080) and the texture combine;
   - what Mdlts_plus's three vec2 (value / velocity / acceleration, slots 0/1/5) drive (GUESS: a
     UV scroll);
   - the other parameter fields (Mdl `+0x10` f32, `+0x15`, `+0x18`, `+0x19` read at
     `0x589e63..0x58a058` in FecMdl; Mdlts `+0x28..` ranges and `+0x4a`), and the `_plus` trailing
     field (`+0x1a` / `+0x4b`);
   - culling, depth test/write and sort order (Lmatrix bytes 4/5 -> `0x5bc450`).

   These parts are 10 of 20 in the dive SHO0 and 8 of 16 in the slam SHO0.
2. **MDL rendering details.** The exe's reader of the vertex type (sub `+0`) and the primitive
   (group `+0`) was not traced, so 9.4 is INFERRED from data. Also unknown: the unit scale
   (16384 = 1.0, INFERRED); the mesh f32 fields; the colour tables (mesh `+0x0c` / `+0x14`) and
   group `+0x04` per-vertex tables (mesh `+0x10` > 1; vertex-colour animation? GUESS); the
   texture-ref flags (7) and which texture a multi-texture MDL binds per mesh.
3. **Blend and sampler state.** The D3D state behind blend code 0xA0600 (`0x5c02a0` -> 2), used by
   every shock model and sprite. Also the address mode (texobj `+0x24/+0x28` = 2 set at
   `0x5fd06f/0x5fd076`, GUESS wrap; this matters for scrolling UVs), the filtering
   (`+0x2c` = 1, `+0x30` = 3) and premultiplication (the decoders output straight alpha).
4. **Space and attachment.** FEP units against KH1 units; the y/z negation (INFERRED FEP +y = down);
   where DDD starts SHO0 (the effect matrix from the attachment `+0x1a8`, `0x5ae260`, and the
   EAD/ESE notes) and whether it follows the bone. Also the part matrix types used here (0, 1, 3, 4):
   `0x5b10e0` modes, the branches of `0x5b1510`, and the decoder's note that types 0/1/3/4/5 differ
   outside the code read.
5. **Time base.** The unit of `0x6294e0` (GUESS: one FEP frame = 1/30 s). It sets how fast the rings
   grow and fade.
6. **Colour scale and fade.** Whether colour 128 = 1.0 (GUESS), and how the stop fade `+0x148`
   (AJC SHO0 has stop mode 0) reaches the draw.
7. **HD textures.** The runtime ordinal used by `0x679e90`/`0x67a1a0` (INFERRED: section-2/frr order),
   how `0x678130` uploads a DDS (format, mips, sRGB), and whether the 2x/4x art changes any
   UV-dependent look.
8. **Anm_plus draw** (`0x586f10`) for the flash sprites (AJC SHO0 parts 0-3, AKD SHO0 parts 0, 1, 4,
   6). Open: billboarding, the quad vertex reading of 9.5 (INFERRED), and the key flag 0x80 and
   frame flags 0x80/0x88.
9. **Megaanm_plus** (153; its own code `0x58cc40`, `0x58cdd0`, `0x58e010`, `0x58e7f0`): the sparkles
   (AJC SHO0 parts 11, 17; AKD SHO0 parts 14, 15).
10. **Generator details.** Child placement and direction for Generateanchor (tail of `0x5b23b0`,
    random cube), the child matrix types, and why one of part 16's blocks has child -1.
11. **The dive's trail**: Shapetail + Locus_tail_strategy + Null_routegenerator (AJC SHO0 part 18,
    spawned by Generateanchorvertex part 19), FecGenerateanchorvertex itself and the VTL lists it
    reads.
12. **Everything else the 19 elements need**:
    - Bindlocus_polygon + Wave_bindlocus_strategy (the CAL0 trails, COM LOC1/LOC2, BOS LOC1 and the
      Keyblade trail W_010 LOC0), including who calls slot 8;
    - Colorblink / Colorblink_parameter, Ellipse_line_particle, Radialg_line_particle;
    - Generate's rate/interval/chance logic;
    - the Rnd* offset tables;
    - the loop rewind's class-id matching (13 looping parts).
13. **Format leftovers, not needed to draw.** None of these occur in or affect the Flowmotion
    elements, as far as checked:
    - header byte 3 and `+0x1a`; element `+0x04/+0x08/+0x0c/+0x20/+0x30/+0x92`; body `+0x02/+0x08/
      +0x0b/+0x28`; part `+0x10/+0x11/+0x12`; signature fields a, b, c;
    - section 6 and body `+0x30` (never present);
    - part ref bit 13 (states 0x40/0x80);
    - the category-mask semantics (4);
    - key lists past the lifetime (6.3);
    - the bytes past param_size in classes 68/80/119;
    - the four unreferenced dispatchers `0x5b6240`, `0x5b6540`, `0x5b68a0`, `0x5b69b0` (INFERRED dead
      code);
    - who registers resident.frr's entries.

## 14. Corrections to the topic specs

The topic specs were left as their authors wrote them. These are the skeptics' corrections (and the
synthesis's own finds, marked *new*):

**structure.md**
- Part `+0x10` is non-zero in 3 Flowmotion parts (not 0 in 355/355); `+0x12` is 1 in all 37328
  parts (not "nearly all"). The dumper's combined observation hid this; it is now split (15).
- The slot-list rule holds game-wide for 7 of the 8 lists; the draw list may be empty (361 parts).
- `_plus` parameter blocks = base layout + a trailing field (not just the base layout).
- The category test exists in two forms (`0x5b4311` value AND, `0x5b4717` bit index).
- Offset-table entries are relative to the owning body/part; spawn event `+4` to the event;
  `0x5ab330` sets bits 16..23 and relocates only when bits 16 and 18 are clear.
- Missing: id-0 resource records are skipped; second signature check `0x5af620`; no code reads
  param_size/work_size; `+0x15` decides the draw-state allocation; spawn events must ascend.
- ESE: entries 16-aligned with 0xCD padding; 10 names missing from their FEP; 34 stale indices.
- Section 13's element table lists only the non-basic classes, without counts, and has small
  slips (COM LOC2 has Generateanchor, not Generate). Section 11 here is the complete list.

**commands.md**
- 42 element bodies in the seven files, not 46 (355 parts is right).
- `ctx` bit 0 filters only slot 2 (to ids 14/132); slot 7 does not run; slot 3 is not filtered.
- The part-death release does not set `ctx+0x78/+0x80/+0x88`.
- The first key list does not always restate every command (11 Flowmotion parts).
- Loop rewind matches keys by **class id** with an init-list fallback, and overwrites `+0x46` per
  slot (5.2 was wrong; structure.md had it right).
- Spawn-abort clean-up always uses the failing record (not "only if out of order").
- The rebase `0x5aadd0` has now been read (structure.md 9); the 4th magic byte is not checked by
  `0x5ab930`.
- Lmatrix byte 0 = matrix type holds in Flowmotion only (122 blocks elsewhere differ).
- *new*: FecMegaanm_plus (153) has its own vtable (`0x7eb2c0`: `0x58df30`, `0x58cc40`, `0x58cdd0`,
  `0x58e010`, `0x58dfe0`, `0x58e7f0`); 6.9's RVAs (`0x58bc30`, `0x58a940`, `0x58aad0`) are FecMegaanm
  (21), which no Flowmotion file uses.
- Four more dispatchers exist (`0x5b6240`, `0x5b6540`, `0x5b68a0`, `0x5b69b0`, no callers);
  `ddd_fep_slotcalls.py` reports only the first call per table reference (misses `0x5b52bd`,
  `0x5b60f6`, `0x5b61e3`).

**textures.md**
- **HD replacement exists** (9.3): 9 of the 18 Flowmotion textures are replaced in the PC game.
- MDL `+0x10/+0x14` are the texture-reference list (not unknown); `+0x20` is its first entry.
- The managers' classes are a direct store (`0x5552b0`, `0x5b8740..0x5b8780`), not inferred.
- The local id formula is complete (9.1).
- The bytes after `0xF000001F` are the first ANM, not a TEX.
- ANM texture entry `+4` is the runtime slot. Key field 0 is 0x80 in 9 of 203 keys. Frame `+0` is
  8, 0x88 or 0x80.
- *new*: MDL vertex format = PSP GE vertex type per sub (9.4); the "16-byte vertex" is only type
  0x11e. ANM quad vertices read as u, v / 16384 (9.5).
- `tools/ddd_fep_survey.py` (documented in textures.md) was overwritten by mistake and is now a
  placeholder. `tools/ddd_fep_skeptic_res.py` checks sections 2..5 over all 667 FEPs instead.
- `ddd_fep_textures.py` uses `w//8` tiles per row where the exe uses `(w+7)>>3`. This matters only
  for widths that are not a multiple of 8, and there are none in Flowmotion.

**ddd-notes.md** ("Visual effects: the FEP format") is outdated: see structure.md section 16 (header
`+0x18` is the category; sections 0/1 are groups/bodies; 6/7 are special elements/names; `+0x48`
is the signature table; an entry's first u16 is a slot index; the ESE is matched by name).

## 15. Tools (all in `tools/`, each with a docstring; run with the venv python and `-I`)

| tool | what |
|---|---|
| `ddd_fep_dump.py <fep> [<ese>] [--no-params] [--gaps]` | the parser: whole tree, raw parameters, element summary, coverage, CHECKS, PROBLEMS. Outputs for the 7 files in `D:/KH-Extract/ddd/fep_dump/`. **Fixed** in this pass: the observation "part +0x10 == 0 and +0x12 == 1" is now two checks (the skeptic showed the combined one hid +0x12 == 1 everywhere); the dumps were regenerated, and only those CHECKS lines changed |
| `ddd_fep_dump_survey.py [root]` | the parser over all 667 FEPs: 0 crashes, 0 problems, 0 uncovered non-zero bytes (re-run after the fix) |
| `ddd_fep_fm_classes.py [--parts] [--draw] [--all]` | *new*: section 11 (per-element classes, union) and, with `--draw`, each draw command's resource with its textures and blend codes |
| `ddd_fep_mdlshape.py <fep> [index ...]` / `--frr <id> ...` | *new*: MDL geometry summary using the vertex-type layout (9.4) |
| `ddd_fep_parts.py`, `ddd_fep_check.py` | the dispatch decoder's parser / checks (they crash on the 23 FEPs with null part entries; not on Flowmotion) |
| `ddd_fep_fn.py`, `ddd_fep_vtables.py`, `ddd_fep_methods.py`, `ddd_fep_slotcalls.py` | disassemble functions / class slots, vtables, override sets, table call sites |
| `ddd_fep_textures.py`, `ddd_fep_frr.py`, `ddd_fep_grid.py`, `ddd_fep_consts.py`, `ddd_fep_firstpix.py` | texture decoding to PNG (exact), resident.frr dump, contact sheets, stack constant tables, decoder probes |
| `ddd_fep_sigcheck.py`, `ddd_fep_ptr.py`, `ddd_fep_vtname.py`, `ddd_fep_basrefs.py`, `ddd_fep_lindis.py`, `ddd_fep_cooccur.py`, `ddd_fep_grep.py`, `ddd_fep_vt.py` | exe helpers |
| `ddd_fep_skeptic_*.py` | the skeptics' independent checkers (structure over 667 files, resources, dispatch, slot scans) |
| `ddd_fep_survey.py` | placeholder only (overwritten by mistake; see 14) |
