# FEP resources: textures, models, sprite animations, index tables

DDD (KH 2.8 Steam build), `effect/**/*.fep` and `effect/resident/bin/resident.frr`.
Worked out from the files and the exe (image base 0x140000000; all addresses below are RVAs).
Labels: unmarked = checked (in the exe at the RVA given, and/or on the data of **all seven**
Flowmotion FEPs: `sys/ffa/bin/p_fa_ajc0`, `p_fa_akd0`, `p_fa_ask0`, `p_fa_com0`,
`sys/pcm/bin/p_ac_com0`, `pc/ac/bin/p_fa_bos_0`, `wep/ex/bin/w_010_so`);
**INFERRED** = follows from checked facts but not seen directly; **GUESS** = a guess.

Tools (in `tools/`, each has a docstring):

| tool | what |
|---|---|
| `ddd_fep_textures.py` | decodes TEX/CTRT exactly as the exe does and writes PNGs: `ddd_fep_textures.py [fep ...]` (default: the 7 Flowmotion FEPs) to `D:/KH-Extract/ddd/fep_png/<fep>/s2_<slot>_<id>.png` + `index.txt`; `... resident` writes all 32 resident textures to `fep_png/resident/<id>.png`; `--sheet`, `--no-resident`, `--out`; `selftest` checks the RGB565/RGBA4 bit expansion against the exe's formulas |
| `ddd_fep_survey.py` | checks every 24-byte record of FEP sections 2..5 (id type bits, index, magic, resident presence) and prints TEX/CTRT fields; `all` = every FEP (field statistics at the end) |
| `ddd_fep_frr.py` | dumps resident.frr's tables |
| `ddd_fep_grid.py` | a labelled RGB/alpha contact sheet of a PNG folder (for looking at them) |
| `ddd_fep_consts.py` | recovers the constant tables a function builds on its stack (Morton, ETC1 tables) |
| `ddd_fep_firstpix.py` | prints the first per-texel block of each PICA decoder |
| `ddd_fep_grep.py`, `ddd_fep_vt.py` | regex over the disassembly; vtable / qword / string dump |

Exported: `D:/KH-Extract/ddd/fep_png/{p_fa_ajc0,p_fa_akd0,p_fa_ask0,p_fa_com0,p_ac_com0,p_fa_bos_0,w_010_so,resident}/`
(52 Flowmotion texture slots - resident ones included, decoded from resident.frr - 18 distinct
textures, + all 32 resident ones), grid views in
`fep_png/_view/`. All were looked at: glows, a radial streak burst, rings/arcs, lightning, noise,
cloud wisps, a rainbow arc, a 256x256 sprite atlas (stars, sparkles, crescent, smoke puffs,
lightning sprites, symbols), digits, a slot-machine atlas, music notes. Text in them ("?Z!",
"0..5", "The Grid" in a non-Flowmotion FEP) reads upright with the first data row at the top.

---------------------------------------------------------------------------------------------

## 1. Sections 2..5 of the FEP: resource references

The FEP header's counts at `+0x20/+0x22/+0x24/+0x26` and table offsets at `+0x30/+0x34/+0x38/+0x3C`
(sections 2, 3, 4, 5; file offsets of u32 tables of file offsets) give 24-byte records:

| +off | type | meaning |
|---|---|---|
| 0x00 | f32 | 1.0 in every record of the 7 files (0.95 in two other FEPs). Not read by the registration code. Meaning unknown. |
| 0x04 | u32 | resource id (0 = empty slot) |
| 0x08 | u32 | size in bytes of a local resource (0 for resident / empty). Not read by the registration code. |
| 0x0C | u32 | file offset of a local resource (for resident/empty records it just repeats the next local offset or the name block: meaningless) |
| 0x10 | i32 | -1 in the file; **runtime**: the manager slot the resource was registered in (written at 0x5b30ef / 0x5b0b1f..., reset to -1 at 0x5b0bd1) |
| 0x14 | u32 | 0 |

**What each section is** (checked in the exe and on the data):

| section | count / table | magic of local resources | manager (in the effect system object) | registered by |
|---|---|---|---|---|
| 2 | `+0x20` / `+0x30` | `TEX\0` texture | `+0xB8`, CTextureManager (vtable 0x7eab18; 192 CEffTexture slots of 0x118 bytes) | 0x5b3020 |
| 3 | `+0x22` / `+0x34` | `MDL\0` model | `+0xB0`, MDL manager (key at MDL `+8`) - CMdlManager (vtable 0x7ed718) INFERRED | 0x5b2b50 |
| 4 | `+0x24` / `+0x38` | `ANM\0` 2D sprite animation | `+0xA8`, CAnm2DManager (vtable 0x7eabb8) INFERRED | 0x5b0a50 |
| 5 | `+0x26` / `+0x3C` | `VTL\0` index lists | `+0xC0`, CFermdlManager (vtable 0x7eac58) INFERRED | 0x5b0f10 |

- FEP registration 0x5ab930: checks bytes `F`,`E`,`P` only (byte 3 varies: 0, 1, 6, 7... across
  files), u16 `+4` == 1 and u16 `+6` == 9 (constants at 0x7ed6a8/0x7ed6ac), `[+0x18] &= 0xF`,
  increments a counter (the FEP's owner number), calls 0x5ab330, then registers sections
  2, 4, 3, 5 (0x5ab993..0x5ab9bd). Header `+0x14` = the file's size (all 7 files).
- Per record (0x5b3020 for TEX; the other three are the same code with another count/table/manager):
  id 0 -> skipped; **`(id & 0x78000000) == 0x70000000` -> resident**: the manager's vtable+0x28
  (0x55abb0 -> 0x55ae10) finds the slot holding that id with owner 0; **otherwise local**: vtable+0x20
  (0x55abd0) finds `(owner, id)` or else (0x55ac20) takes a free slot, calls the resource object's
  Load (its vtable+0x20) on `FEP + [rec+0x0C]`, keys the slot by the id read back from the data
  (vtable+0x40: TEX `+4` 0x5c0330, ANM `+4` 0x55a470, MDL `+8` 0x5beed0, VTL `+4` 0x5c0a00),
  stores the owner, refcount+1. The slot index goes to `rec+0x10`.
- Find 0x55ae10 matches a slot whose owner is the caller's **or 0**, so resident resources sit in
  owner-0 slots (INFERRED: registered at start-up from resident.frr; the frr is loaded at 0x557f09
  via 0x618970, the code that registers its entries was not found).

**Ids.** Local: `base | type << 7 | index`, type 0 TEX, 1 ANM, 2 MDL, 3 VTL, index = the record's
position in its section (checked for every local record of the 7 files), base from the FEP
(`0x9803AC00` for p_fa_ajc0 whose header `+0x10` is `0x33AC0000`; `0x98039800` / `0x33980000`
akd0; `0x9004B800` / `0x24B80000` com0; `0x98089C00` / `0x389C0000` bos_0: the byte `0xAC/0x98/0xB8/0x9C`
is the header id's second byte; the top 16 bits (0x9803, 0x9004, 0x9808) were not decoded).
Resident: `0xF0000000 | (k >> 7) << 26 | type << 7 | (k & 0x7F)` for entry k of resident.frr's
table `type` (checked for all 326 entries; `0xF4000100`/`0xF4000103`/`0xF4000104` = MDL 128/131/132,
used by p_fa_com0 and p_ac_com0). Every resident id in the 7 FEPs exists in resident.frr, and every
record's id has the type bits of its section (`ddd_fep_survey.py`: 0 problems).

## 2. resident.frr (`effect/resident/bin/resident.frr`, 1 266 400 bytes)

| +off | value | meaning |
|---|---|---|
| 0x00 | `FRR\0` | |
| 0x04 | 0x80003800 | unknown |
| 0x08 | 1 | unknown (version? GUESS) |
| 0x0C | 4 | number of tables (resource types) |
| 0x10 | 0x1352DF | file size - 1 (GUESS: it is) |
| 0x14 | u32 x4 | counts: 32 TEX, 134 ANM, 136 MDL, 24 VTL |
| 0x24 | u32 x4 | offsets of four u32 tables, every entry 0xFFFFFFFF (INFERRED: runtime slots, like FEP `rec+0x10`) |
| 0x34 | u32 x4 | offsets of four u32 tables of file offsets of the resources (TEX, ANM, MDL, VTL in that order) |

Each resource is the same block as in an FEP (`TEX`/`ANM`/`MDL`/`VTL`, id at `+4`, MDL at `+8`).

## 3. TEX and CTRT: the texture

```
TEX (at the record's offset)
+0x00 "TEX\0"
+0x04 u32 id
+0x08 u16 1, u16 1            (all 805 local TEX of all FEPs; meaning unknown)
+0x0C u32 0                   runtime: set to 1 when loaded (0x5c0401..0x5c0407)
+0x10 i32 -1                  < 0: upload from the CTRT (0x5c0417); >= 0 seen only in
                              rs/nd/bin/r_nd_hrp_0.small.fep (no CTRT there)
+0x14 u32 1, +0x18 u32 0      (always; unknown)
+0x1C u32 width, +0x20 u32 height   read by 0x5c0310 (a size query, used at 0x55f753)
+0x24 u32 0x80                offset of the CTRT from the TEX (0x5c041d)
+0x28..+0x80 zero
CTRT (TEX + 0x80)
+0x00 "CTRT"   (another loader, 0x54e450, also knows a "CTPK" container; not in FEPs)
+0x04 u32 0
+0x08 u32 0x10                offset of the info block from the CTRT (0x5c042c)
+0x0C u32 0x80                offset of the pixels from the CTRT (0x5c0434)
info (CTRT + 0x10)
+0x00 u32 0                   (0x60 in 145 non-Flowmotion textures: see "replacement" below)
+0x04 u32 data size
+0x08 u32 0
+0x0C u32 format, PICA200 0..13 (0x5fd062; values > 13 become 0)
+0x10 u16 width, +0x12 u16 height   the size the exe decodes and creates (0x5fd03f, 0x671493)
+0x14 u8  mip count (0 everywhere; > 0 would only change sampler fields 0x5fd07d, levels are not decoded)
+0x15 u8  2 (always; unknown), +0x16 u16 0
pixels (CTRT + 0x80), data size bytes
```

**Load path:** CEffTexture::Load 0x5c03f0 -> 0x5fd030(texture, pixels, info, 6, 0) ->
0x5fc2f0 (replacement lookup) -> if none, 0x671460 -> 0x671480: switch on the format
(jump table 0x67193c) to a decoder that returns a new **RGBA8 buffer, byte 0 R, 1 G, 2 B, 3 A,
row 0 = the first data row** -> 0x677e70 creates the GPU texture from it.

**Layout:** 8x8 tiles, row-major from the **top** (tile row 0 is written to output rows 0..7:
0x66dd39..0x66dd4e), inside a tile Morton order with x in bit 0: texel `(x, y)` of a tile is
number `x0 | y0<<1 | x1<<2 | y1<<3 | x2<<4 | y2<<5` (the 64-entry table built at 0x66dc13, read
with `ddd_fep_consts.py`). No vertical flip: the exported PNGs read upright.

**Formats** (decoder RVA; what one texel becomes; checked in each decoder's code):

| fmt | name | bits | decoder | texel -> RGBA |
|---|---|---|---|---|
| 0 | RGBA8 | 32 | 0x66e9c0 | bytes A,B,G,R -> (R,G,B,A) |
| 1 | RGB8 | 24 | 0x66e650 | bytes B,G,R -> (R,G,B,255) |
| 2 | RGBA5551 | 16 | 0x66cf60 | u16: R 15-11, G 10-6, B 5-1, A bit 0 (0/255); 5 bits -> `c<<3 | c>>2` |
| 3 | RGB565 | 16 | 0x66e1c0 | u16: R 15-11, G 10-5, B 4-0; `c<<3|c>>2`, `c<<2|c>>4`; A 255 |
| 4 | RGBA4 | 16 | 0x66cb60 | u16: R 15-12, G 11-8, B 7-4, A 3-0; each `n*17` |
| 5 | LA8 | 16 | 0x66dbb0 | byte0 = A, byte1 = L -> (L,L,L,A) |
| 6 | HILO8 | 16 | 0x66ded0 | (byte1, byte0, 0, 255) |
| 7 | L8 | 8 | 0x66d4e0 | (L,L,L,255) |
| 8 | A8 | 8 | 0x66c8d0 | (0,0,0,A) |
| 9 | LA4 | 8 | 0x66d7d0 | high nibble L, low nibble A, each `n*17` |
| 10 | L4 | 4 | 0x66c610 | even texel in the low nibble; (l*17, l*17, l*17, 255) |
| 11 | A4 | 4 | 0x66c370 | same nibble order; (0,0,0,a*17) |
| 12 | ETC1 | 4 | in 0x671480 (0x67160e) + 0x671b00 | see below, A 255 |
| 13 | ETC1A4 | 8 | in 0x671480 (0x67173b) + 0x671b00 | see below |

**ETC1 / ETC1A4.** An 8x8 tile is 4 blocks in the order top-left, top-right, bottom-left,
bottom-right (order table 0x03020100 at 0x671764, output steps 0x67181e/0x67182b). ETC1A4 block =
8 bytes alpha (u64 LE, texel `(x, y)` at nibble `x*4 + y`, `a*17`) then 8 bytes ETC1; ETC1 block =
8 bytes. The ETC1 block is read as two LE u32: `lo` at +0 (u16 at +0 = LSB plane, u16 at +2 = MSB
plane, bit `x*4+y`), `hi` at +4 (standard ETC1's upper word: colours, table codewords bits 7-5 and
4-2, diff bit 1, flip bit 0); modifier table (2,8)(5,17)(9,29)(13,42)(18,60)(24,80)(33,106)(47,183)
(read from 0x671b2a..). **The exe does not bit-replicate the base colours** (0x671c67..0x671c9f,
0x66ef15..0x66efbb): individual mode `c4 << 4`, differential mode `c5 << 3` and
`(c5 << 3) + delta*8` (delta table 0,1,2,3,-4,-3,-2,-1), then `clamp(base + modifier, 0, 255)`.
A standard ETC1 decoder (`c*17`, `c<<3|c>>2`) is up to 15 levels brighter; `ddd_fep_textures.py`
follows the exe (option `std` in `etc1_block` for the standard one).

**Replacement.** 0x5fc2f0 compares the 0x50 bytes before the pixels (CTRT+0x30..0x80) with a
zeroed buffer (0x5fc374, `call 0x771bf8`: memcmp INFERRED): equal -> no replacement (0x5fc37b). Otherwise, with info+0 == 0x60 and a name at
CTRT+0x60 (each byte + 0x16, e.g. `uvfire03`, `beam`, `sibuki`), it looks the name up (a list at
`[obj+0x670]`) for another texture (INFERRED: the remaster's HD replacements). **Every Flowmotion
texture and every resident texture has those bytes all zero, so the game uses exactly the decoded
pixels** (the 145 named textures are all in other FEPs). The exporter notes a name when present.

**Size quirks.** Resident `0xF000000F` (RGBA4) and `0xF000001F` (LA8): TEX says 128x128, CTRT says
128x80, data is 0x4000 bytes (= 128x64). The exe decodes 128x80, so rows 64..79 come from the
bytes after the texture (the next TEX header and pixels): visible garbage in the bottom 16 rows of
their PNGs. `0xF000000F` (a swirl in its top-left 64x64) is used by p_ac_com0 (slot 3). In other
FEPs 53 local textures are not power-of-two and some "TEX/CTRT dims differ" or run past their
record size; none of that occurs in the Flowmotion files.

## 4. The Flowmotion textures

Only one texture is stored in a Flowmotion FEP: p_fa_ajc0's slot 6 `0x9803AC06`, 128x64 L8 (a white
wedge/fan). Everything else is resident:

| id | size | format | content | used by (slot) |
|---|---|---|---|---|
| F0000001 | 128x64 | RGBA4 | streaky noise (alpha) | all 7 |
| F0000004 | 128x64 | LA8 | flame/feather tongue | ajc0 5, akd0 2, ask0 4, com0 9, ac_com0 1 |
| F0000005 | 128x64 | LA8 | quarter disc + soft round glow | w_010_so 6 |
| F0000006 | 128x64 | ETC1A4 | rainbow arc segment | com0 11, ac_com0 7, w_010_so 7 |
| F0000007 | 128x128 | LA8 | radial streak burst (quarter, corner = centre) | all 7 |
| F0000008 | 128x128 | RGB565 | soft star burst, alpha 255 | ajc0 8, com0 5 |
| F0000009 | 128x128 | LA8 | gradient wedge | w_010_so 8 |
| F000000C | 128x64 | LA8 | ring arc (quarter ring) | ask0 2, com0 4, ac_com0 5, w_010_so 5 |
| F000000F | 128x80 (see quirks) | RGBA4 | swirl | ac_com0 3 |
| F0000012 | 128x64 | LA8 | leaf, small glows | akd0 5 |
| F0000015 | 256x256 | LA8 | sprite atlas | all 7 |
| F0000016 | 128x64 | LA8 | wisps, dots, glow, square | com0 8 |
| F0000018 | 128x64 | RGBA4 | horizontal streaks | ajc0 4, com0 10, ac_com0 6 |
| F000001B | 128x64 | RGBA4 | wavy band | ajc0 7 |
| F000001C | 128x64 | RGBA4 | caustic web | akd0 6, com0 7 |
| F000001D | 128x64 | LA8 | twisting smoke/lightning column | ajc0 2, akd0 3, com0 2, bos_0 3 |
| F000001E | 128x64 | ETC1A4 | iridescent blobs | com0 1 |

(w_010_so slots 3, 4 are empty.) Every texture id that a Flowmotion MDL (`+0x20`) or ANM (`+0x3C`)
names is in its FEP's section 2 (checked, all 7 files), so section 2 is the list of textures the
effect uses.

## 5. MDL (section 3): models

CMdl::Load 0x5bf7a0 (vtable 0x7ed6e0 [4]) turns file offsets (relative to the MDL start) into
self-relative ones; the fields it touches give the structure:

```
MDL
+0x00 "MDL\0", +0x04 u16 1, u16 10, +0x08 u32 id, +0x0C u32 0 (runtime loaded flag, 0x5bf7cc)
+0x10 u32 1, +0x14 u32 0x20 (unknown)
+0x18 u32 mesh count, +0x1C u32 offset of the meshes (0x44 bytes each)       (0x5bf7d3, 0x5bf7f4)
+0x20 u32 texture id (0 = none)  +0x24 i32 -1  +0x28 u32 7 (43 and 47 in the two untextured ones)
+0x2C u32 blend code (0xA0600; 0x70600 in 0xF000012E)  +0x30 0  +0x34 -1     (from the data)
mesh (0x44): +0x04, +0x0C, +0x14, +0x18 offsets (relocated), +0x10 n, +0x1C sub count,
   +0x20 offset of subs (16 bytes each)                                          (0x5bf80e..0x5bf854)
sub (16): +0x08 count, +0x0C offset of groups (16 bytes each)                     (0x5bf87a..0x5bf895)
group (16): +0x04 offset, +0x0C offset (relocated); more when mesh+0x10 > 1      (0x5bf8ba..0x5bf8e3)
```

From the one small local model looked at (p_fa_bos_0 `0x98089D02`, 0x350 bytes; INFERRED):
mesh+0x04 -> s16 x,y,z positions (16), mesh+0x0C/+0x14 -> u32 colours 0x80808080 / 0x00808080,
mesh+0x24..+0x2C and +0x38..+0x40 f32 (a size and a centre? GUESS); group: `+0` 4 (primitive type?
GUESS), `+4` -> one u32 per vertex (0xEC/0xF0/0xF4: offsets into the colour table), `+8` vertex
count (14, 13), `+0xC` -> vertices of **16 bytes: u16 u, u16 v (0x1000 = 1.0? GUESS), u32 colour,
s16 x, y, z, u16 0x01EE**; the x,y,z repeat the position table. Scale of positions and UVs and the
primitive type are not established.

The models and animations the Flowmotion FEPs register (resident ids without the `0xF000` prefix;
`L` = local; `-` = empty slot), by slot:

| FEP | section 3 MDL | section 4 ANM | section 5 VTL |
|---|---|---|---|
| p_fa_ajc0 | 12E 13E 11A 13D L(AD04) 158 145 - 140 L(AD09) L(AD0A) L(AD0B) L(AD0C) L(AD0D) L(AD0E) | 0AB L(AC81) L(AC82) 087 0A3 0B6 0A7 L(AC87) | 186 L(AD81) |
| p_fa_akd0 | 118 L(9901) L(9902) L(9903) 13D 179 12E 13E 102 17C L(990A) L(990B) L(990C) L(990D) L(990E) | 0AB L(9881) 0CA 080 087 L(9885) | 186 |
| p_fa_ask0 | - 157 171 14F 10C 168 151 10D | 0AB 0D5 | |
| p_fa_com0 | - 16E - 110 171 14A 14F 150 14C L(B909) 158 F4000100 F4000103 170 106 16D 117 152 157 10B 148 11D 118 - 15B 10D 168 L(B91B) L(B91C) 159 166 | 0AB 0D4 0A3 - 0A5 0B6 087 0F3 0D3 | |
| p_ac_com0 | 11A - 11E 133 119 157 156 F4000104 164 101 152 15C 10E 174 150 10B 137 14D 159 147 | 0AB | |
| p_fa_bos_0 | - 158 L(9D02) | 0AB 0D4 0A3 | |
| w_010_so | 157 - 171 - - 17C - 146 144 14A | 0AB 0D5 0D9 09E 0A3 0B2 0D4 0B4 | |

## 6. ANM (section 4): 2D sprite animations

CAnm2D::Load 0x55a5f0 (vtable 0x7eab80 [4]):

```
ANM
+0x00 "ANM\0", +0x04 id, +0x08 u16 1, u16 2, +0x0C u32 0 (runtime loaded flag, 0x55a609)
+0x14 u32 total length = sum of the key durations (32 = 16 x 2, 38 = 19 x 2; INFERRED)
+0x18 u32 count (1), +0x28 u32 offset (0x3C) -> {u32 texture id, u32 ?}   (0xF0000015 etc.)
+0x1C u32 count, +0x2C u32 offset -> keys of 12 bytes {0, frame index, 2 (duration? GUESS)}
+0x20 u32 count, +0x30 u32 offset -> frames, 12 bytes {u32 8?, u32 n, u32 offset of n u32 quad
       indices}; the offset is relocated (0x55a633)
+0x24 u32 count, +0x34 u32 offset -> quads, 0x1C bytes {u32 0, u32 colour? (0x80315998...),
       u32 4 (vertices), u32 offset of the vertices (relocated 0x55a661), u32 blend code
       0xA0600 (read at 0x55a152 -> 0x5c02a0), u32 0, i32 -1}
+0x38 f32 (49.125, 185.375, ...; unknown); the texture list follows at +0x3C
```

Vertices of a quad: 4 x 10 bytes, read as `u16, u16, s16, s16, u16 0` (for example `0x0840,0x0040,
-6879,6879,0`). Reading the first pair as a UV did **not** land on single cells of the 256x256
atlas with either 1/4096 or 1/8192 per unit (`fep_png/_view/anm_uv.png`), so the vertex layout is
an open question. Blend codes (0x5c02a0): 0x70600 -> 1, 0xA0600 -> 2, 0xA0602 -> 3, else 0.

## 7. VTL (section 5): index lists

The Load of the section-5 manager's objects is `ret 0` (0x5c0af0, CFermdl vtable 0x7eac20 [4];
INFERRED that this is the section-5 class: its key is read at `+4` like VTL's, and the manager
sits next to the others). Observed (ajc0 `0x9803AD81`, resident `0xF0000180/1`):

```
VTL: "VTL\0", id, u16 1, u16 2, u32 0, u32 0, u16 0, u16 list count, u32 0x1C (offset of the lists)
list: u32 n, u32 offset (from the VTL) of n u16 values
```
e.g. ajc0: one list `0,1,2,3,4`; `0xF0000181`: `0,2,4..30, 1,3..31`, `0..63`, ... (INFERRED:
vertex orderings for strips/tails; GUESS which command uses them.)

## 8. What a port needs

- Textures: the PNGs from `ddd_fep_textures.py` hold exactly the RGBA8 the exe builds (straight,
  not premultiplied: no decoder multiplies by alpha); row 0 = top = v 0 in D3D (INFERRED from the
  upload of the decoded buffer as-is at 0x677e70). LA8 sets RGB = L, L8 sets A = 255, A8 sets RGB = 0.
- Resolve ids: local id -> the FEP's own resource; `0xF0/0xF4` ids -> resident.frr.
- Open: MDL vertex/primitive semantics, ANM vertex layout and timing, the samplers (wrap/clamp,
  filtering) and the blend states behind the codes 0x70600/0xA0600/0xA0602.
