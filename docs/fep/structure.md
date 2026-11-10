# The FEP file: structure from the header down to the command entries

DDD (KH 2.8, Steam build), `effect/**/*.fep` (+ `.ese`). Read from the files and the exe only
(image base 0x140000000; every address below is an RVA). Companion documents in this folder:
`textures.md` (sections 2..5 and the TEX/MDL/ANM data they point to) and `commands.md` (what the
command classes do with their parameters).

Labels: unmarked = checked, in the exe at the RVA given and on the data of **all seven**
Flowmotion FEPs (`sys/ffa/bin/p_fa_ajc0`, `p_fa_akd0`, `p_fa_ask0`, `p_fa_com0`,
`sys/pcm/bin/p_ac_com0`, `pc/ac/bin/p_fa_bos_0`, `wep/ex/bin/w_010_so`). Most structural rules
were also checked on **all 667 FEPs** of the game (section 12). **INFERRED** = follows from
checked facts but was not seen directly. **GUESS** = a guess.

## 0. Tools and outputs

| tool (in `tools/`) | what |
|---|---|
| `ddd_fep_dump.py <fep> [<ese>] [--no-params] [--gaps]` | the parser: prints the whole tree (header, names, groups, elements with their ESE sound, bodies, spawn events, parts, slots, init list, key lists with every entry's raw parameters in hex + f32, the 8 slot lists, resources), then an element summary, a parameter-size check, byte coverage, every structural rule of this document tested on the file (passes / failures), and problems. Bounds-checks every offset; prints `!!` where something is out of range or not decoded |
| `ddd_fep_dump_survey.py [root]` | runs the parser over every `.fep` under `root` and totals crashes, problems, checks, uncovered bytes |
| `ddd_fep_sigcheck.py <fep>... [--records]` | compares each file's command signature table with the exe's (0x9e7d10) and lists the per-id records |
| `ddd_fep_methods.py [--owner rva]` | the 9 vtable methods of every command class; `--owner` says which class/slot a code address belongs to |
| `ddd_fep_ptr.py`, `ddd_fep_vtname.py`, `ddd_fep_basrefs.py`, `ddd_fep_lindis.py`, `ddd_fep_cooccur.py` | exe helpers: vtables/pointers, RTTI name of a vtable, image-base-relative table accesses, linear disassembly to a text file, co-occurring instruction patterns |

Dumps of the seven Flowmotion FEPs: `D:/KH-Extract/ddd/fep_dump/<name>.txt` (`p_fa_ajc0.txt` ...
`w_010_so.txt`). In all seven: 0 problems, every non-zero byte of the file lies inside a parsed
structure, and every rule below holds.

## 1. How a .fep becomes runtime data

1. **Resource type.** The extension table at `0x9ea0f0` (16-byte records `{u64 code, char *ext}`)
   has `0x800a -> "fep"` (`0x9ea180`) and `0x100d -> "ese"`: FEP is resource type 10, ESE type 13.
2. **After loading**, the resource system switches on the type (`0x6169d1`, jump table
   `0x616e18` / index bytes `0x616e80`): type 10 goes to `0x616cce -> 0x558300 -> 0x5ab930`
   (with the effect manager `[0xaa3cd0]`).
3. **`0x5ab930` registers the file:** checks the bytes `F`,`E`,`P` (byte 3 is not checked), s16
   `+4 == [0x7ed6a8]` (1) and s16 `+6 == [0x7ed6ac]` (9); masks header `+0x18` to its low 4 bits;
   increments manager `+0x30`; relocates (`0x5ab330`); then registers the resources:
   textures `0x5b3020`, 2D animations `0x5b0a50`, models `0x5b2b50`, fermdl `0x5b0f10`.
4. **Relocation `0x5ab330`:** if header `+0x0c` bit 16 is set it does nothing (already done);
   else it sets bits 16..23 there and calls `0x5aadd0(body, file)` for every section-1 body.
   `0x5aadd0` adds `file - struct` to every non-zero 32-bit offset of the body, its tables and
   the structs below it (all listed in sections 6-9), so **in the file every 32-bit offset is
   file-relative, and after loading every offset under section 1 is relative to the struct that
   holds it** (0 stays 0 = none). Offsets in the header, in the section tables and in sections
   0, 2..7 are not relocated; the code adds the file base to them (`0x5ac274`, `0x5abb83`,
   `0x5acc75`, `0x5b3053`). The 16-bit parameter offsets of command entries are relative to their
   list in the file already and are never touched.
5. **Starting an effect by name** (e.g. `0xfc305`: `sprintf("%s.fep")` `0x64ce0`, lookup
   `0x6186f0`): `0x5567d0` gets the FEP (type 10) and ESE (type 13) references
   (`RsrcReferenceT<FEP_DATA>`, vtable `0x7878e0`; `RsrcReferenceT<EFFECT_SE>`, `0x7eaab8`),
   finds the element by name in section 7 (`0x5abb70`), and `0x5559a0` -> `0x54fc80` ->
   `0x5acb40` turns the name's id into a group and element (section 3), and `0x5ac210` creates the
   element instance from the group's body (section 1).
6. **Creation checks (`0x5ac210`):** for each command id the body lists (`+0x24`), the file's
   signature table must agree with the exe's (`0x5ac2c2..0x5ac30a`); and an element started at a
   frame at or past the body's length is not created (`0x5ac30c`).

## 2. Header (0x48 bytes)

| off | type | meaning |
|---|---|---|
| 0x00 | char[3] + u8 | `"FEP"` + a byte that varies (0..65 over the 667 files; not checked; **GUESS**: a save counter) |
| 0x04 | s16 | 1 (version; must be 1, `0x5ab959`) |
| 0x06 | s16 | 9 (version; must be 9, `0x5ab965`) |
| 0x08 | u32 | 0x48: offset of the command signature table header (section 4) |
| 0x0c | u32 | 0 in the file; runtime flags (bit 16 = relocated, `0x5ab330`) |
| 0x10 | u32 | **id base**: bits 17-31 of every element id of the file (bits 0-16 are 0 in all 667 files); returned by `0x5aadc0` |
| 0x14 | u32 | file size (equal in all 7) |
| 0x18 | u16 | `&= 0xf` at load (`0x5ab971`); equals `id base >> 28` in all 667 files (the element category, section 7); 3/3/3/2/2/3/4 for ajc0/akd0/ask0/com0/p_ac_com0/bos/w_010 |
| 0x1a | u16 | 0 in all 667 files; no reader found |
| 0x1c | u16 | count of section 0 (groups) |
| 0x1e | u16 | count of section 1 (bodies) (`0x5ab3af`) |
| 0x20..0x26 | u16 x4 | counts of sections 2 (textures), 3 (models), 4 (2D animations), 5 (fermdl) |
| 0x28 | u32 | section 0: offset of u32[count] offsets of group records |
| 0x2c | u32 | section 1: offset of u32[count] offsets of bodies |
| 0x30..0x3c | u32 x4 | sections 2..5: offsets of u32[count] offsets of 24-byte resource records (section 11) |
| 0x40 | u32 | section 6: offset of `{u16 count, u16 (garbage), u32 offset -> u32[count] record offsets}` |
| 0x44 | u32 | section 7: offset of `{s32 count, u32 offset -> 12-byte name records}` |

The header is followed by the signature table, then (in the files' order) groups, bodies,
resource records, elements, names' records, the bodies' contents, the embedded resource data,
and the name strings at the end. The parser does not rely on this order.

## 3. Element ids, names (section 7) and groups (section 0)

**Element id** (u32): `id base | group number << 10 | element number`, bits 0-8 = element
number, bit 9 = "special" (section 6), bits 10-16 = group number, bits 28-31 = category
(`0x5b4717`: `id >> 28` (arithmetic) is a bit index into the manager's mask `+0x2c`; a set bit
hides/skips the element).

**Names (section 7)**, 12-byte records: `u32 element id, u32 name length, u32 file offset of
the name` (NUL-terminated). `0x5abb70` compares length then text and returns the record's
**index**. Every name of the 7 files (and of all 667) resolves to an element. The name order is
not the group order (p_fa_com0: `P_FA_COM_FOR2_0` is name 8 but group 12).

**Id -> element (`0x5acb40`):**
- bit 9 set: look through section 6 for a record whose u32 `+0` equals the id, then `0x5b6f30`.
  Section 6 is empty in all 667 files; its record layout was not read.
- else: find the section-0 record whose `(id >> 10) & 0x7f` equals the id's group number
  (`0x5acc80..0x5acc9f`), then in its element table the element whose `id & 0x1ff` equals the
  id's (`0x5acce2..0x5acd03`), and create it with body = the record's `+4` (`0x5acd39`).

**Group record (section 0), 12 bytes:**

| off | type | meaning |
|---|---|---|
| 0x00 | u32 | id of the group (element number 0); its group number is what `0x5acb40` matches |
| 0x04 | u16 | body index (section 1) used by all of the group's elements |
| 0x06 | u16 | number of elements |
| 0x08 | u32 | file offset of u32[n] file offsets of elements |

In the 7 files every group has one element and every body one group.

## 4. Command signature table (header +0x08)

`{u32 count = 158, u32 offset}` at 0x48, then 158 records of 8 bytes, one per command id 0..157:
`u8 a, u8 b, u16 c, u16 param_size, u16 work_size`. The table is **byte-identical to the exe's
table at 0x9e7d10** (count 158 at `0x9e7d00`) in all 7 files (`ddd_fep_sigcheck.py`). At creation
`0x5ac2f2` compares `u32[file_table + id*4]` with `u32[0x9e7d10 + id*4]` (note: indexed by 4,
not 8; equal tables pass either way).

- `param_size` = bytes of the command's parameter block: checked on the layout (each block's room
  up to the next structure is >= it, with only zero bytes after it) for all classes except
  FecLocus_tail_strategy (68), FecShapetail (80) and FecDistortional_model (119), whose blocks
  carry non-zero bytes past it (940 / 689 / 165 blocks over the 667 files; `ddd_fep_dump.py`
  reports them; not decoded).
- `work_size` = the command's work area in 2-byte units: every slot's work offset + work_size
  <= its part's `+0x02` (213768 slots), and in p_fa_ajc0 part 0 the slots' work offsets are the
  running sums (0x00 Posac 0x40, 0x40 Rot 0x20, 0x60 Scl 0x20, 0x80 Colac 0x40, 0xc0 Lmatrix 0,
  0xc0 Wave 4 -> 0xd0 (rounded), Polygon 0x20 -> 0xf0 = part `+0x02`).
- ids 152..157 (`FecAnm_plus`, `Megaanm_plus`, `Mdl_plus`, `Mdlts_plus`, 156, `FecMaskModel`)
  have all-zero records. **INFERRED**: the `_plus` classes use the parameter layout of
  Anm/Megaanm/Mdl/Mdlts: their blocks' room is always 0x20 / 0x118-0x120 / 0x1c-0x20 / 0x4c-0x50
  bytes against 0x1f / 0x115 / 0x1a / 0x4b (all 667 files). The parser sizes them by that room.
- `a`, `b`, `c`: meaning unknown (**GUESS**: field counts of the parameter and work layouts).

## 5. Element (0x9c bytes; per group)

The element instance keeps a pointer to it at `+0x1a8`.

| off | type | meaning |
|---|---|---|
| 0x00 | u32 | element id (its group bits equal the group record's: 4145/4145) |
| 0x04 | u16 | 1 in all 42 Flowmotion elements; no reader found |
| 0x06 | u8 | view-angle test (`0x5b4a0b..0x5b4f66`): 0 off; 2/3 compare the direction to the camera with an axis (mode 3 rotates it by `+0x60/+0x64/+0x68`, `0x5b4b60..0x5b4b9a`) and hide or fade by the thresholds `+0x74`, `+0x84` (**INFERRED** fade). 0 in all 42 |
| 0x07 | u8 | -> instance `+0x1bc` |
| 0x08 | u16 | not read by the code found |
| 0x0a | u8 | view-bounds test (`0x5b4903`): 0 = built-in bounds (`0x9e83b0`, `0xaa3df0`), 2/3 = extent `+0x40` / centre `+0x50` through `0x5bbcb0`, others none. 2 in all 42 |
| 0x0b | u8 | distance test (`0x5b49c2`, `0x5bb350`/`0x5bb0f0`): 0 = distances 124/240, 4 = 70/115 (tables `0x9e8358`/`0x9e8340`), 2/3 = `+0x94`/`+0x98`, 1 = none; the result scales the alpha or hides (**INFERRED** fade between the two). 0 in 41, 1 in 1 |
| 0x10 | vec4 | added to the start position (`0x5ac67b..0x5ac6ce`) |
| 0x20, 0x30 | vec4 | no reader seen in the code read (0x20 is (0,-0,-0) in the 7 files; 0x30 is (1,1,1) in 40 of the 42 elements, (1.2,1.2,1.2) and (1.5,1.5,1.5) in two: **GUESS** rotation and scale) |
| 0x40 / 0x50 | vec4 | bounds extent / centre for `+0x0a` 2/3 |
| 0x60..0x68 | f32 x3 | angles for `+0x06` mode 3 |
| 0x74, 0x84 | f32 | thresholds of the view-angle test (`0x5b4cf7`, `0x5b4d1a`) |
| 0x8e | u8 | -> instance `+0x1be` |
| 0x90 | u16 | start delay: the instance clock starts at `-value` (`0x5ac4eb`) |
| 0x94, 0x98 | f32 | the two distances of `+0x0b` modes 2/3 (300, 500 in all 42) |

Vec4 `w` fields and some unused words hold garbage from the authoring tool (e.g. `0x01f7e5e8`).

## 6. Body (section 1, 0x34 bytes)

An element instance points to its body at `+0x1a0` and to the file at `+0x198`.

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | flags -> instance `+0x04` (`0x5ac496`). When the clock reaches the length (`0x5b44ac`): bit 1 = looping effect: its parts with part flag bit 1 loop (`0x5b4424`), and it lives until a stop request (instance `+2` bit 0x10) with no parts left; bit 0 = lives until no parts are left (`0x5b456f`); neither = removed. Flowmotion: 0 (18 bodies) or 2 (24) |
| 0x02 | u16 | 0 in all 42; no reader found |
| 0x04 | s32 | length in frames -> instance `+0x16c` (`0x5ac530`) |
| 0x08 | u16 | 0/30/60/90/120/128/240 in the 42; no reader found (**GUESS**: frames) |
| 0x0a | u8 | -> instance `+0x1b7`; picks which manager counter counts it (`0x5ac449`: 4, 5-8/11, other). 0 in all 42 |
| 0x0b | u8 | 0 in all 42; no reader found |
| 0x0c | u32 | on a stop request (`0x5b43a7`): 0 = fade out over 30 frames (`0x867efc`), 1 = over 15 (`0x867ef0`), 3 = ignore it and run on, other = remove at once (`0x5b4518`). 0 or 3 in the 42 |
| 0x10 | u16 | number of spawn events (`0x5b45f6`) |
| 0x12 | u16 | number of parts |
| 0x14 | u16 | number of command ids at `+0x24` |
| 0x16 | u16 | 12 in all 4145 bodies (= the 12 records at `+0x28`, **INFERRED**) |
| 0x18 | u16 | number of part groups at `+0x2c` |
| 0x1a | u16 | number of `+0x30` records: 0 in all 4145 bodies |
| 0x1c | s32 | -> s32[n] spawn events |
| 0x20 | s32 | -> s32[n] parts (an entry 0 = no part, `0x5b7428`; 17 such in 8 non-Flowmotion files) |
| 0x24 | s32 | -> u16[n] command ids used: contains every slot's id of every part (4145/4145), no duplicates; checked against the signature table at creation |
| 0x28 | s32 | -> s32[12] -> 8-byte records `{u32 count, s32 -> u16[count]}`, relocated as 2 x 6 (`0x5ab0b2..0x5ab227`); no reader found. Flowmotion: mostly empty, e.g. p_fa_ajc0 body 0 record 0 = 4: [0, 1, 1, 2] |
| 0x2c | s32 | -> s32[n] part groups (below) |
| 0x30 | s32 | -> s32[n] records whose `+8` is an offset (relocated, `0x5ab2ac`); never present |

**Spawn event**: `s16 frame, s16 n, s32 -> u16 part ref[n]`. When the element clock reaches the
frame, each ref starts a root part (`0x5b45b2..0x5b4693` -> `0x5b73d0`); the instance keeps the
next event's index at `+0x1b0`.

**Part ref** (u16, also in generator parameters): bits 0-11 = part index in this body's part
table (`0x5b740b`); bit 13 = a flag the spawner sets on the parent and the child (`0x5b752d`);
bit 12 (generator parameters only) = the low bits are a part-group index instead
(`0x5b0e80`). Spawn-event refs use only bits 0-11 and 13 (20575 refs).

**Part group** (`+0x2c`): `u16 count, u16 (garbage), s32 -> 4-byte records {s16, u16 ref}`.
`0x5b0e80` returns the count, `0x5b0ec0` the ref of record j; a generator whose part ref has bit
12 spawns each record's part (`0x57fd1d`, `0x5800b9`). In every file each body has one group
listing its parts as `(spawn frame, file-wide part number)` - the part's index plus the parts of
all earlier bodies (37489 records) - with one `(x, 0xffff)` record in later bodies. Those refs
would index past the body's own table for every body but the first, and no generator in the
Flowmotion files uses a bit-12 ref (**INFERRED**: this group is authoring data the game does not
use; a generator that did use it would need refs relative to the body).

## 7. Part (0x48 bytes)

A part is a particle/object template; instances are spawned by spawn events and by generator
commands (`0x5b7980`, from FecGenerate `0x581ef6`, FecGeneratevertex `0x580eb8`,
FecGeneratevertex_roty `0x581845`, FecGenerateanchorvertex `0x580067`, FecGenerateanchor
`0x580679`). The part instance (`0x5b73d0`) keeps the part at `+0x58`.

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | flags -> instance `+0x04`: bit 0 = after its life, live on while it has children (`0x5b4542`); bit 1 = loops between `+0x0c` and `+0x0e` if the body loops too (`0x5b441b`, `0x5b5040`) |
| 0x02 | u16 | work area size in 2-byte units (instance work = `2*v` bytes at instance `+0xa0`, `0x5b7446`, `0x5b75bf`) |
| 0x04 | u16 | lifetime in frames -> instance `+0x44` (`0x5b758a`); the instance dies when its clock passes it (`0x5b4535`) |
| 0x06 | u16 | the part's own index in the body (37328/37328) |
| 0x08 | u8 | number of slots (commands) |
| 0x09 | u8 | bit 0 picks the element's part list: set -> `+0x178` (list 0), clear -> `+0x180` (list 1) (`0x5b750c`). List 1 is updated by `0x5b4700`, which the per-frame pass `0x5b47a0` calls (`0x5b4d3c`) only for elements that passed their view/distance tests (and `0x5ac7b7` at creation); list 0 by `0x5b5350`/`0x5b53e0`, without those tests. All 355 Flowmotion parts have 2 (list 1) |
| 0x0a | u8 | 0 in all 37328 parts |
| 0x0b | u8 | matrix mode -> instance `+0x90` (`0x5b766a`): 2 and 6..9 get three 0x40-byte matrices, 0xff none, others two (`0x5b749f..0x5b74e6`). Flowmotion: 0, 1, 3, 4, 6, 255 |
| 0x0c | u16 | loop start frame (`0x5b6779`) |
| 0x0e | u16 | loop end frame (`0x5b443c`, `0x5b50c2`) |
| 0x10 | u8 | 0 in 355 of 355 Flowmotion parts but not everywhere; no reader found |
| 0x11 | u8 | equals the lifetime's low byte in all 37328 parts; no reader found |
| 0x12 | u8 | 1 in nearly all parts; no reader found |
| 0x13 | u8 | number of key lists (`0x5b52dc`) |
| 0x14..0x1b | u8 x8 | counts of the 8 slot lists at `+0x28..+0x44` |
| 0x1c | s32 | -> s32[`+0x08`] slot descriptors |
| 0x20 | s32 | -> init list (command list, below) |
| 0x24 | s32 | -> s32[`+0x13`] key lists |
| 0x28..0x44 | s32 x8 | -> u16[] slot lists (below) |

**Slot descriptor** (12 bytes + inputs):

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | command id: class object `[0x9e7450 + id*8]` (`0x5b57b3`); names in `docs/ddd-fec-commands.txt` (line id-1) |
| 0x02 | u16 | work offset in 2-byte units: work = instance work + `(2*v) & 0xffff` (`0x5b5782..0x5b57a4`) |
| 0x04 | u16 + u16 | number of inputs n (the second u16 is 0 in all files; `commands.md` reads it as s32) |
| 0x08 | s32 | -> `u16 slot[n]` then `u16 work[n]`: the other slots this one reads and their work offsets (every one equals that slot's own) |

Example (p_fa_ajc0 body 0 part 0): Lmatrix reads Posac, Scl, Rot; Wave_bindlocus reads Lmatrix;
Bindlocus_polygon reads Colac, Lmatrix, Wave_bindlocus.

**The 8 slot lists** say which slots take part in each engine phase. Each list is exactly the
set of the part's slots whose class overrides that vtable method (2840 lists of the 7 files, all
but 2 draw lists of `p_ac_com0` body 5 parts 3 and 5, which leave out their FecMdlts_plus):

| list | count | vtable | called | by |
|---|---|---|---|---|
| +0x28 | +0x14 | +0x10 update | every frame after the keys | `0x5b5f10` phase 2 (`0x5b61e3`, `0x5b6502`, `0x5b6642`) |
| +0x2c | +0x15 | +0x18 draw | when drawn; two passes split by the byte table `0x9e7c60` bit 0 (set only for FecLensflare_light) | `0x5b59d0`, `0x5b5c20` (`0x5b5b92`, `0x5b5dc0`) |
| +0x30 | +0x16 | +0x20 release | part death | `0x5b82b0` (`0x5b83c6`) |
| +0x34 | +0x17 | +0x28 loop | the part loops | `0x5b6680` (`0x5b6811`), after `0x5b80c0` finds each slot's key at the loop start |
| +0x38 | +0x18 | - | - | empty in all 37328 parts; no reader found |
| +0x3c | +0x19 | +0x30 first | once (instance flag 4) | `0x5b5f10` phase 7 (`0x5b60f6`), `0x5b53e0` (`0x5b5535`) |
| +0x40 | +0x1a | +0x38 | phase 8 | `0x5b5f10` phase 8 (`0x5b604d`), from `0x5b5e70` |
| +0x44 | +0x1b | +0x40 | no dispatch through the command table found | (only Wave_tail/Wave_bindlocus override it) |

`vtable +0x00` (init) and `+0x08` (key) have no list: init runs for every init-list entry and key
for every key-list entry.

## 8. Command lists and entries

The init list and every key list have the same layout (12-byte header, entries right after it,
parameter blocks after the entries: 147852 lists):

| off | type | meaning |
|---|---|---|
| 0x00 | s16 | frame: the list runs when the part clock reaches it (`0x5b51dc`, `0x5b5309`); 0 for the init list |
| 0x02 | s16 | the smallest span among its entries (110524/110524); 0 for the init list; not read by the code found |
| 0x04 | s32 | number of entries |
| 0x08 | s32 | -> entries (8 bytes each) |

**Entry** (8 bytes):

| off | type | meaning |
|---|---|---|
| 0x00 | u16 | slot index (into the part's slot table and the instance's param/entry arrays) |
| 0x02 | u16 | parameter block offset from the list header (`0x5b5786`, `0x5b5271`); the block is `param_size` bytes (section 4). Blocks usually start 16-aligned but not always (p_fa_com0 `0xba26`: a FecLmatrix block right after a 0x36-byte FecRndcol block), so a reader must use the offsets, not a running sum; for a command with param_size 0 the offset is 0 or points past the entries |
| 0x04 | s16 | start frame = its key list's frame (329749/329749); local time t = part clock - this (`0x5b5b45`, `0x5b6027`, `0x5b61a8`) |
| 0x06 | s16 | span = (frame of the same slot's next key, else the part's lifetime) - start (329749/329749; -1 when a key lies past the lifetime). Passed to the next key of that slot (`0x5b523d`) |

The init list holds one entry per slot, in slot order, all with frame 0 and span 0 (37328
parts). The first key list is always at frame 0; it usually but not always restates every slot.

**How they run** (`0x5b56f0`, `0x5b5180`):
- At spawn (`0x5b56f0`): for each init entry, `instance+0x68[slot] = list + param offset`, then
  `vtable +0x00(manager, params, work)`; returning 0 aborts the spawn and calls `+0x20` for the
  entries done (`0x5b57e8..0x5b5863`).
- Each frame (`0x5b5180`, from `0x5b5040`): while the next key list's frame <= the part clock,
  for each entry: `vtable +0x08(manager, new params, work, old params, f32 old span)` - old span
  = the slot's previous entry's `+6`, or the clock if none (`0x5b5230..0x5b52bd`); then
  `instance+0x68[slot]` = the new params and `instance+0x70[slot]` = the entry. The instance's
  next key index is at `+0x46` (-1 after the last).
- The phase methods get t = part clock - current entry's `+4` (as f32) where they take a time
  (update, draw, phase 8, loop).
- Looping (`0x5b5040`): when the clock passes `+0x0e`, the keys up to `+0x0e`-1 run, `0x5b6680`
  re-applies each loop-list slot's key at `+0x0c` (`vtable +0x28`), and the clock becomes
  `(clock - end) mod (end - start) + start`. `0x5b80c0` matches slots by **command id**, not
  slot index (two slots of one class would get the later one's key).

## 9. What the relocation (0x5aadd0) touches

Under a body E (offsets made relative to E): `+0x1c` table and each spawn event's `+4`; `+0x20`
table and, for each part P (offsets made relative to P): `+0x1c` table (count `P+8`) and each
slot's `+8`; `+0x20` and the init list's `+8`; `+0x24` table (count `P+0x13`) and each key
list's `+8`; `+0x28..+0x44`. Then E `+0x24`; `+0x28` and its 12 records' `+4`; `+0x2c` table
(count `+0x18`) and each group's `+4`; `+0x30` table (count `+0x1a`) and each record's `+8`.
(`0x5aadd0..0x5ab325`.) This list is also the complete list of offset fields under section 1.

## 10. Runtime instances (for reference)

- Element instance: allocated by `0x5bd400` in `0x5ac210`, 0x1d0 bytes; `+0x04` body flags,
  `+0x138` clock, `+0x148` alpha, `+0x16c` length, `+0x178`/`+0x180` part lists (head/tail at
  `+0x188`/`+0x190`), `+0x198` file, `+0x1a0` body, `+0x1a8` element, `+0x1b0` next spawn event,
  `+0x128` next element instance.
- Part instance (`0x5b73d0`): `+0x02` flags, `+0x04` part flags, `+0x18` next, `+0x20` parent,
  `+0x30` children, `+0x40` clock (f32), `+0x44` lifetime, `+0x46` next key, `+0x58` part,
  `+0x60` work, `+0x68` params per slot, `+0x70` current entry per slot, `+0x78/+0x80/+0x88`
  matrices (by `+0x0b`), `+0xa0` work area.

## 11. Resources (sections 2..5)

24-byte records `{f32, u32 id, u32 size, u32 data offset, s32 handle (-1 in the file, set at
load), u32}` (details in `textures.md`). The registering functions (`0x5b3020`, `0x5b0a50`,
`0x5b2b50`, `0x5b0f10`) pass `id` to the handler's vtable `+0x28` when `id & 0x78000000 ==
0x70000000` (a shared resource by id), else the data at the offset to vtable `+0x20`; the result
goes to `+0x10`. Handlers (manager field, RTTI of the object stored there):

| section | handler | object |
|---|---|---|
| 2 | manager `+0xb8` | `SQEXTEC::CTextureManager` (vtable `0x7eab18`), data magic `TEX` |
| 3 | manager `+0xb0` | `SQEXTEC::CMdlManager` (`0x7ed718`), `MDL` |
| 4 | manager `+0xa8` | `SQEXTEC::CAnm2DManager` (`0x7eabb8`), `ANM` |
| 5 | manager `+0xc0` | `SQEXTEC::CFermdlManager` (`0x7eac58`), `VTL` |

(set at `0x55567b..0x5556c2` through `0x5b8740..0x5b8780` from the globals `0xaa3c98..0xaa3cb0`.)
Commands refer to them by index (e.g. FecGenerateanchorvertex reads `fermdl[+0x10]` through the
file at element instance `+0x198`, `0x57fca5..0x57fcd5`).

## 12. The ESE

Matched to the FEP **by name**, not by position: on first use `0x559cf0` looks up each ESE
entry's name in the FEP's section 7 (`0x557bd0` -> `0x5abb70`), stores the index at the entry's
`+2` and sets ESE `+8` bit 0. The ESE lists only elements that have a sound (p_fa_com0: 6 of
20; p_ac_com0: 6 of 8); p_fa_bos_0 and w_010_so have none.

| off | type | meaning |
|---|---|---|
| 0x00 | char[4] | `"ESE\0"` |
| 0x08 | u32 | runtime flags |
| 0x0c | u16 | 2 in all four; unknown |
| 0x0e | u16 | number of entries |
| 0x10 | s32[n] | entry offsets from the file start |
| entry +0x00 | u16 | number of sounds |
| entry +0x02 | s16 | FEP name index; as stored it already equals the name's section-7 index (18/18) |
| entry +0x04 | char[16] | element name |
| entry +0x14 | n x {char[32] sound name, u32} | the u32 is 30 (10 for ASK MLC0); meaning not read |

## 13. The Flowmotion elements (from the dumps' summaries)

| element | file | body | frames | parts | command classes besides Pos/Rot/Scl/Col and their mv/ac, Lmatrix |
|---|---|---|---|---|---|
| AJC CAL0 | p_fa_ajc0 | 0 | 35 | 6 | Generateanchorvertex, Bindlocus_polygon + Wave_bindlocus_strategy, Anm_plus, Mdl_plus, Megaanm_plus |
| AJC SHO0 | p_fa_ajc0 | 1 | 45 | 20 | Generateanchor(vertex), Shapetail + Locus_tail_strategy + Null_routegenerator, Rndrot, *_plus |
| AKD DRL0 / CAL0 / SHO0 | p_fa_akd0 | 0/1/2 | 30/30/45 | 11/5/16 | Generate, Rndpos/rot/scl / Generateanchorvertex, Bindlocus_polygon / *_plus |
| ASK MLC0 | p_fa_ask0 | 0 | 30 | 19 | Ellipse_line_particle, Colorblink(+_parameter), Generateanchor, *_plus |
| COM WAL0 | p_fa_com0 | 11 | 20 | 14 | Generateanchor, Rndcol/pos/rot, *_plus |
| COM BSS0 | p_fa_com0 | 12 | 30 | 13 | Generate, Generateanchor, Radialg_line_particle, Colorblink, *_plus |
| COM LOC0 / LOC1 / LOC2 | p_fa_com0 | 9/10/19 | 40 | 2/7/3 | LOC0: Shapetail + Locus_tail_strategy + Null_routegenerator; LOC1, LOC2: Bindlocus_polygon + Wave_bindlocus_strategy (+ Generate) |
| COM MLC0 / FIR0 | p_fa_com0 | 16/17 | 20 | 11/18 | Generateanchor, Rndrot, *_plus |
| AC MSL0 / MSS0 / RNG0 | p_ac_com0 | 0/1/6 | 30 | 2/7/5 | Rndrot, Generateanchor, Mdl(ts)_plus, Anm_plus |
| BOS BSS0 / LOC1 | p_fa_bos_0 | 0/1 | 30/40 | 6/5 | Colorblink, Generateanchor / Bindlocus_polygon + Wave_bindlocus_strategy |
| W_010 LOC0 (Keyblade trail) | w_010_so | 5 | 6 | 1 | Bindlocus_polygon + Wave_bindlocus_strategy |

The full per-element class lists are at the end of each dump ("ELEMENT SUMMARY").

## 14. Checks

`ddd_fep_dump.py` tests these rules on each file and `ddd_fep_dump_survey.py` totals them. Over
all 667 FEPs: 0 crashes, 0 problems, no non-zero byte outside a parsed structure, and every rule
holds - header `+0x18` = id base >> 28; groups' body index in range; element ids match their
group; every name resolves; spawn refs in range and only bits 0-11/13; part-group refs file-wide
(37489); work offset + work size <= part work (213768); entries right after the 12-byte list
header (147852); parameters after the entries (508191); init lists = one entry per slot in order,
frame 0, span 0 (37328); key frames ascending (73196); key `+2` = smallest span (110524); entry
frame = key frame and span rule (329749); first key at frame 0; part `+0x06` = own index; `+0x11`
= lifetime & 0xff; `+0x0a` = 0; list `+0x38` empty; body `+0x16` = 12; body id list without
duplicates and containing every slot id. Observations that do **not** always hold: the first key
list sets every slot (2386 parts do not); the body id list equals the slots' ids (17 bodies list
extra ids, beside empty part entries); part `+0x10` = 0 and `+0x12` = 1 (2912 parts differ).

## 15. Not understood (left raw in the dumps)

- Header byte 3, `+0x1a`; element `+0x04`, `+0x08`, `+0x0c`, `+0x20`, `+0x30`, `+0x92`; body
  `+0x02`, `+0x08`, `+0x0b`; part `+0x10`, `+0x11`, `+0x12`; signature fields a, b, c.
- Body `+0x28` (12 u16 lists) and `+0x30` (never present); section 6 records (never present);
  part list `+0x38` (always empty).
- The bytes past `param_size` in FecLocus_tail_strategy / FecShapetail / FecDistortional_model
  blocks.
- The meaning of each command's parameters: see `commands.md`; this document and the dumps give
  them raw.
- ESE: the u32 after each sound name.

## 16. Corrections to `ddd-notes.md` ("Visual effects: the FEP format")

- `+0x18` holds the category (3/2/4), not 1; `+0x14` is the file size, `+0x10` the id base.
- Section 0 records are **groups** (id, body index, element count, element table), not elements;
  the elements (0x9c bytes) hang under them.
- Section 1 records (0x34 bytes) are **bodies** (timeline, parts), shared by a group's elements.
- Sections 6 and 7 are not offset tables: 6 = special elements (empty), 7 = the name table.
- `+0x48` is not part of the header: it is the command signature table header `{158, offset}`.
- A command entry's first u16 is a **slot index**; the command id is in the slot descriptor.
- The ESE is matched by name; its entries carry the FEP name index and only exist for elements
  with a sound.
