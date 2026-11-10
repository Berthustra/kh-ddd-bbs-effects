# FEP aura: DDD's Flowmotion aura (EAD motion "001", group 200)

Scope: the purple glow Sora (and the Keyblade) carry while in Flowmotion: what starts it, which
events of the pseudo-motion `001` run, how it ends, the command classes of its elements, and the
one class not decoded before (FecFlash). Exe: DDD Steam (KH 2.8), RVAs (image base 0x140000000).
**Labels:** unmarked = read in the exe or the files; **INFERRED** = follows from what was read, a
link not traced; **GUESS** = not read. Placement / clock / stop mechanics are space-time.md's and
are only referenced here.

Tools: `tools/ddd_fep_aura_events.py [--group 200] [--md]` (emulates the start filter `0x5528b0`
on both EADs and prints what starts, with placement case), `tools/ddd_fep_aura_callers.py <rva>`
(call sites grouped by enclosing function, with the states each sets),
`tools/ddd_fep_aura_exe.py <rva>...` (string / u32 / f32 at an RVA), plus the existing
`ddd_fep_fm_classes.py --all`, `ddd_fep_parts.py`, `ddd_fep_fn.py -c FecFlash`.

---

## 0. The answer in one paragraph

On every **fresh** entry into the Flowmotion state 27 (`0x2c3cd0` called with edx = 0, any
command except 145 NightmareCharge / 146 NightmareFinish), the player calls the aura-slot setter
`0x2bcb60(player, 200, slot 0)`, which starts **every event of motion `001` with group 200 (or
negative) and no end time, all at once**, on Sora's EAD (`[player+0x130]+0x88`) and on the
Keyblade's (`[[player+0x5c8]+0x130]+0x88`), and stores 200 in player `+0x6a0`. That is 18 events
on Sora (FOR0, FOR1, FSH0 on `mune`; ORA0 x8 on legs and arms; ORA1 x7 on thighs, hands, head,
pelvis) and 1 on the Keyblade (W_010_SO_ORA0_0 on `wep`). All 19 have motion-change mode 2
(**keep**): motion changes inside Flowmotion do not touch them. They end when the player leaves
Flowmotion through the common "back to a normal state" routine `0x2a7b80`, which calls
`0x2bcb60(player, 0, slot 0)`: a **stop request** (no kill) to every looping keep-mode effect
whose start group was 200, on both controllers. The looping bodies (stop mode 3) then stop
looping and end when their parts have died; FSH0 (not looping) is a one-shot that has normally
ended long before.

## 1. Start

### 1.1 The start-all call `0x553a80` -> `0x5528b0`

`0x553a80(ctrl, name, group)`: `0x551a80` finds the motion index by name in the controller's EAD
(`[ctrl]`), then `0x5528b0(ctrl, index, group)`:

```
if ctrl+0x60 == 0 or ctrl EAD == 0 or no event list (0x5519a0): return
if group <= -2: return                                          0x5528e5
for each event e of the motion (52 bytes, file order):           0x552910..0x552972
    if group >= 0: if not (e.group < 0 or e.group == group): continue
    else:          group = e.group     # quirk for -1: later events filtered by the first one's group
    if e.t1 (+0x2c) != 0: continue                                0x552926  (t0 is NOT tested)
    0x552990(ctrl+0x60 records, e, group, level = byte ctrl+0x4c, ctrl+0x4a, ctrl+0x4d,
             owner = ctrl+0x38, ctrl+0x50, time 0)
```

`0x552990` (space-time 1, 3.1): skips the event when its level (`flags >> 12`) is above the
level byte (`0x5529c2`); resolves the bone; takes the first free slot of the controller's record
table (`[ctrl+0x60]` = {records, capacity}; a slot is free when its handle id is 0 or its handle
is dead, `0x552a40..0x552aa3`; **table full -> the event is not started**); starts the effect
(`0x5521e0`) and writes the 12-byte record: `+0` handle, **`+4` = the group passed (0 if
negative)** (`0x552aee..0x552afa`), `+6` = e`+0x2a & 0xfff`, `+8` = e`+0x30 & 0xfff`, `+0xa` =
e`+0x32`. No event-state / max-starts / start-time logic is involved: this path is separate from
the per-frame test `0x5545a0`.

### 1.2 The aura-slot setter `0x2bcb60(player, group, slot)`

Two slots: slot 0 = player `+0x6a0`, slot != 0 = `+0x6a2` (`0x2bcb6f..0x2bcb84`).

```
p = slot ? &player.6a2 : &player.6a0
if group > 0:
    if *p > 0 and *p != group: 0x2bcb60(player, 0, slot)        # stop the old one   0x2bcba3
    if *p == group: return                                       # already on         0x2bcbb0
    if player.59c & 0x100000: return     # suppressed (see 1.4); slot NOT written     0x2bcbb6
    0x553a80([player+0x130]+0x88, "001", group)                  # Sora               0x2bcbe3
    0x553a80([[player+0x5c8]+0x130]+0x88, "001", group)          # Keyblade           0x2bcc16
    *p = group
else:                                                            # stop               0x2bcc2e
    if *p > 0:
        0x5ca080([player+0x130], *p, 1)    -> 0x553e40(ctrl, *p, mode 1, kill 0)      0x2bcc4a
        0x2eeaa0([player+0x5c8], *p, 1)    -> same on the Keyblade (twice)           0x2bcc63, 0x2bcc7d
        *p = 0
```

`"001"` is the string at `0x785158` (every player call site loads it).

### 1.3 Who calls it with group 200

| site | function | condition | group, slot |
|---|---|---|---|
| `0x2c3dc6` | `0x2c3cd0` = enter Flowmotion state 27 (sets state 0x1b, `0x2c3d07`) | edx (the "continue" flag) == 0 **and** `(u16)(cmd - 0x91) > 1`, cmd = player `+0x84c` (just copied from `+0x6b2`), i.e. not 145 NightmareCharge / 146 NightmareFinish (`0x2c3da9..0x2c3dbb`) | 200 (`lea edx, [rcx+0x37]`, rcx = 0x91), slot 0 |
| `0x2aa692` | `0x2aa580(player, 0, ..)` = end of the suppress mode (1.4) | player state (`+0x570`) == 0x1b | 200, slot 0 (tail jump) |

Callers of `0x2c3cd0` with edx = 0 (fresh entry, aura starts): the command dispatch
`0x2a0820`, `0x2a29c3` (both store the command in `+0x6b0`/`+0x6b2` first); with a computed edx:
`0x2bdf6f`, `0x2be557`, `0x2c8e99`, `0x2c9abd` (`sete`), `0x2e1e01`. With edx = 1 (chaining
from inside Flowmotion; the aura call is skipped and the aura stays on): `0x2c0448`, `0x2c08f9`,
`0x2c0d5b`, `0x2c1490`, `0x2c260b`, `0x2c2ce1`, `0x2c2d2c`, `0x2c30ec`, `0x2c36b2`, `0x2e2ea6`,
`0x2e3d0c`. Even on a fresh entry the setter is a no-op while slot 0 already holds 200.

So the aura is on for **every Flowmotion action** (dash, wall run, pole, rail, glide, kick,
throw, ...): it is tied to state 27, not to a motion or a command record. The per-motion groups
of triggers.md (105..255) do not matter here: `001` is started with the fixed group 200.

Other callers of `0x2bcb60` (not Flowmotion): slot 1 with group `0x190 + table byte` (400+:
`0x2a041f`, `0x2aa66e`, `0x2e23c6`, `0x2e9a1c`, `0x2e9add`, `0x2ebf97`) and 409 (`0x2bf99f`);
motion `001` has no event with those groups in Sora's EAD (they start nothing for Sora,
**INFERRED** they serve other characters' EADs). Stops of slot 1: `0x2a033f`, `0x2aa6ce`,
`0x2ebc85` (state 0x1d entry).

The status auras (S_SP_HST_ORA0_0 grp 173, S_SP_RER_ORA_0 176, S_SP_BTL_ORA0_0 177, also in
`001`) use a separate slot, player `+0x780`, driven by the 8 status timers at `+0x75c`
(`0x2a4ca0`, table `0x9d4c22`, 20-byte rows, groups only in rows 3, 6, 7) and `0x2a51a4..0x2a51ed`;
their stop is `0x2bc980(player, group, 1)`. Not part of the Flowmotion aura.

### 1.4 Suppression (player `+0x59c` bit 20)

`0x2aa580(player, on, f)`: on -> sets `+0x59c` bit 20, sets bit 0x40 of `+0x64` of the objects at
`+0x128` of the player, its Keyblade and `+0x5d8`, and **stops both aura slots**
(`0x2aa6c0`, `0x2aa6ce`); off -> clears the bit and, **if the state is 27, restarts the aura**
(`0x2aa679..0x2aa692`). While the bit is set `0x2bcb60` starts nothing. **INFERRED**: a "player
hidden / not drawn" mode (cutscene or similar); the KH1 mod can ignore it.

## 2. Stop

### 2.1 What ends it

| site | function | effect |
|---|---|---|
| `0x2a7d7c` | `0x2a7b80(player, kind, ..)` | `0x2bcb60(player, 0, 0)`: stop request, group 200 |
| `0x2aa6c0` | `0x2aa580(player, 1, ..)` (1.4) | same |

`0x2a7b80` has 60 call sites (`ddd_fep_aura_callers.py 2a7b80`), many inside the Flowmotion
code (`0x2c0da0..0x2c3cc7`); it clears the command (`+0x808`, `+0x84c`, `+0x6b2`), resets
action flags, and through `0x2e8360` sets a ground state (1, 2 or 3, `0x2e83bd`, `0x2e8433`) or
calls `0x2c9450` in the air. **INFERRED**: it is the common "action over, back to normal
control" exit, so every normal end of Flowmotion (landing, falling out, cancelling) passes it.
Not traced: whether every *abnormal* exit from state 27 (taking a hit, death, a cutscene, an area
change) goes through it; if one does not, the aura keeps running until the next `0x2a7b80`.

Motion changes do **not** end it: all 19 events have mode 2 (`flags & 3 == 2`, keep), so
`0x552de0` does nothing to them (`0x552e48..0x552e6c`), and it does not empty the record table
(it zeroes `ctrl+0x58`, the event-state count, not the table at `[ctrl+0x60]`; corrects
space-time 5.2's wording "the record table is emptied"). Their records therefore survive every
motion change until `0x553e40` finds them. No end time (t1 = 0), so `0x5545a0` never touches
them either.

### 2.2 The stop routine `0x553e40(ctrl, group, mode, kill)`

```
for each record r in [ctrl+0x60] (12 bytes):                      0x553e90..0x553f08
    if r.handle == 0: continue
    h = 0x5579e0(r.handle); if h == null: r.handle = 0; continue
    if not looping(h) (0x550a20: body flags bit 1): continue          # one-shots play out
    if (r.flags & 3) != 2: continue                                  # only keep-mode records
    if group >= 0:
        mode == 0: skip records whose group == group  (stop all others)
        mode != 0: skip records whose group != group  (stop only this group)
    0x550af0(h, delay 0, tint ramp-out r+0xa)                        # stop request now
    if kill: 0x550440(h, -1)
```

The aura stop passes mode 1, kill 0: **stop request now** to the looping group-200 effects
(the record's group is the group passed at the start, 1.1). The FEP side (space-time 5.1): body
stop mode 3 sets instance `+2 |= 0x10`; parts stop looping, the body ends when its pass lists are
empty. For ORA0/ORA1 that is: each part finishes its current lap to its life (12 frames; the
loop is frames 3..4 and the Colac key at frame 12 takes alpha to 0), i.e. the glow fades out over
at most ~9 FEP frames (0.3 s). FOR0/FOR1: their parts run to their lives (8..20 frames). The
Keyblade's W_010_SO_ORA0_0: the same rule (15-frame body). The EAD fields chg (6), end fade (6)
are never used on this path; fade-in is 0; `max` (1) is not consulted by `0x5528b0`.

### 2.3 Rule for the KH1 mod

- On entering the Flowmotion phase from a non-Flowmotion phase (our `Pending -> *` transition),
  start the 18 Sora events + 1 Keyblade event of section 3, all at once, each at time 0.
- Keep them running across every phase / motion change inside Flowmotion.
- On returning to `Free` (normal control), send each looping one a stop request (body stop mode 3:
  parts stop looping, finish, then the body ends). Do not restart while they are still alive and
  Flowmotion has not been left.
- FSH0 needs no stop: it is a 20-frame one-shot (0.67 s) at the start.

## 3. The events (placement)

`python -I tools/ddd_fep_aura_events.py --md` (Sora `p_ex01_000.ead` motion 001 @0x3308 and the
Keyblade `w_so01_001.ead` motion 001 @0xdf8). All: file `p_fa_com0` (Keyblade: `w_010_so`),
t0 = 0, t1 = 0, group 200, chg 6 (Keyblade 0), end fade 6 (Keyblade 0), fade-in 0, tint 0,
`+0x32` 0, max 1, level 0. Offsets in DDD units (raw s16 = KH1 units), rotations in degrees
applied X, then Y, then Z in the bone's frame (row form `RX·RY·RZ`, space-time 3.2), scale
uniform, RGBA bytes with 128 = 1.0 (alpha 166 = 1.297).

| who | element | bone | offset (DDD) | rot deg (X,Y,Z) | scale | RGBA | flags | case |
|---|---|---|---|---|---|---|---|---|
| Sora | P_FA_COM_FOR0_0 | `mune` | (0.03, -0.02, 0.0) | (0, 0, -70) | 0.75 | 128,128,128,166 | 0x010e | A |
| Sora | P_FA_COM_FOR1_0 | `mune` | (0.03, -0.02, 0.0) | (0, 0, 0) | 0.75 | 128,128,128,166 | 0x0106 | B |
| Sora | P_FA_COM_FSH0_0 | `mune` | (0.03, -0.02, 0.0) | (0, 0, 0) | 1 | 128,128,128,166 | 0x0106 | B |
| Sora | P_FA_COM_ORA0_0 | `L_momo` | (0.25, 0.02, 0.01) | (0, -5, 5) | 1.04 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `L_sune` | (0.02, 0.02, -0.01) | (0, 180, 2) | 0.6 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `L_ude1` | (0.13, 0.0, 0.01) | (0, -12, 0) | 0.7 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `L_ude2` | (0.13, 0.0, 0.0) | (0, 0, 0) | 0.6 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `R_momo` | (0.25, 0.02, -0.01) | (0, 5, 5) | 1.04 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `R_sune` | (0.02, 0.02, -0.01) | (0, 180, 2) | 0.6 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `R_ude1` | (0.13, 0.0, -0.01) | (0, 12, 0) | 0.7 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA0_0 | `R_ude2` | (0.13, 0.0, 0.0) | (0, 0, 0) | 0.6 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `L_momo` | (0.12, 0.02, 0.01) | (0, -5, 5) | 1.2 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `L_te` | (0.08, 0.0, -0.02) | (0, 0, 0) | 0.8 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `R_momo` | (0.12, 0.02, -0.01) | (0, 5, 5) | 1.2 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `R_te` | (0.08, 0.0, 0.02) | (0, 0, 0) | 0.8 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `atama` | (0.09, 0.0, 0.0) | (0, 0, 0) | 1 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `atama` | (0.09, 0.0, 0.0) | (0, 0, 0) | 1.3 | 153,89,153,128 | 0x010e | A |
| Sora | P_FA_COM_ORA1_0 | `koshi` | (0.08, 0.0, 0.0) | (0, 0, 0) | 1.3 | 153,89,153,128 | 0x010e | A |
| Keyblade | W_010_SO_ORA0_0 | `wep` | (0, 0, 0) | (-90, 0, 0) | 1 | 153,89,153,128 | 0x010e | A |

(Exact rotations: raw ±872 = ±0.0872 rad = ±5.0°, ±2094 = ±12.0°, 31415 = 180.0°, 349 = 2.0°,
-12217 = -70.0°, -15708 = -90.0°.)

Flags: `0x010e` = mode 2 keep | bit 2 follow | bit 3 bone rotation | bit 8; `0x0106` = the same
without bit 3. **Case A** (space-time 3.3): node = `[R_e | off] · W_bone(now)` every frame, the
instance basis = normalised `R_e · W_bone`, scale = EAD scale x the bone's row lengths. **Case B**
(FOR1, FSH0): position follows `off · W_mune + t_mune` every frame, axes = world (`R_e` = I),
scale = EAD scale. Bit 8 (`0x5524e4..0x5524ff`, space-time 2): the handle is linked to the owner
and its alpha is multiplied by the owner's `+0x5c` every frame (`0x550f45`) - the aura fades with
the player's own transparency. Colour: handle colour = RGBA/128 (`0x550c00`) -> instance `+0x40`
(x handle alpha, space-time 5.3); the draw classes multiply their Col/Colac output by it. So the
ORA events tint their (white-ish) sprites by (1.195, 0.695, 1.195): purple; FOR0/FOR1/FSH0 keep
their own colours and get alpha x1.297.

The two `atama` ORA1 events (scales 1.0 and 1.3) are both real (two halos on the head). The
Keyblade bone `wep` is at the grip (ddd-notes.md "Keyblade").

## 4. Command classes per element

From `ddd_fep_fm_classes.py --all` ("x n" = commands; body length in FEP frames, 29.97/s).

| element | body | length | loops | stop | parts | classes |
|---|---|---|---|---|---|---|
| P_FA_COM_ORA0_0 | com0 : 0 | 25 | yes | 3 | 3 | Pos x3, Rot x3, Scl x3, Lmatrix x3, Colac x3, Anm_plus x3 |
| P_FA_COM_ORA1_0 | com0 : 1 | 25 | yes | 3 | 1 | Pos, Rot, Scl, Lmatrix, Colac, Anm_plus |
| P_FA_COM_FOR0_0 | com0 : 6 | 20 | yes | 3 | 7 | Pos x6, Posac x1, Rot x7, Scl x5, Sclac x2, Lmatrix x7, Col x1, Colac x5, Generateanchor x1, Anm_plus x4, Megaanm_plus x1, Mdlts_plus x1 |
| P_FA_COM_FOR1_0 | com0 : 7 | 20 | yes | 3 | 3 | Pos x1, Posac x2, Rot x2, Rotac x1, Scl x2, Sclac x1, Lmatrix x3, Colac x1, Generate x1, Rndrot x2, Generateanchor x1, Mdlts_plus x1 |
| P_FA_COM_FSH0_0 | com0 : 18 | 20 | **no** | 0 | 11 | Pos x6, Posac x5, Rot x10, Rotac x1, Scl x7, Sclac x4, Lmatrix x11, Col x1, Colac x8, Generate x1, Rndpos x1, Rndrot x3, Bindlocus_polygon x1, Wave_bindlocus_strategy x1, **Flash x1 (107)**, Generateanchor x3, Anm_plus x2, Megaanm_plus x1 |
| W_010_SO_ORA0_0 | w_010_so : 2 | 15 | yes | 3 | 7 | Pos x7, Rot x7, Scl x7, Lmatrix x7, Colac x7, Anm_plus x7 |

Every class here except **FecFlash (107)** is decoded in README 8, commands.md, generators.md
(Generate, Generateanchor, Rnd*), tails.md (Bindlocus_polygon, Wave_bindlocus_strategy),
draw-sprites.md (Anm_plus, Megaanm_plus), draw-models.md (Mdlts_plus). ORA0, ORA1 and the
Keyblade ORA0 are the simplest possible elements: one Anm_plus sprite per part with keyed
Pos/Rot/Scl/Colac. FSH0's part 10 is the Flash.

Parts (`ddd_fep_parts.py`): ORA0 parts 0-2 and ORA1 part 0: life 12, loop frames 3..4, matrix
type 1, Colac keys 0 / 3 / 4 / 12 (alpha 32 -> 128 -> fade -> 0). FOR0: 7 parts (6 spawned at
frame 0; part 5 by the Generateanchor of part 6), lives 8..14. FOR1: part 2 (life 20,
Generate) spawns parts 0/1. FSH0: spawn event at frame 0 = parts 0, 1, 4, 7, 8, 9, 10; lives
7..16.

## 5. FecFlash (107): a screen-space flash rectangle

Vtable `0x7ecdf0` (RTTI `.?AVFecFlash@FunnyEffectsForPSP@@`): slot 0 `0x596cb0` init, 1
`0x596bb0` key, 2 `0x596be0` update, 3 `0x596da0` draw, 4 `0x596d40` release, 5 `0x597160`
loop, 6-8 defaults. Lists (FSH0 p10): init, key, update, draw, loop.

### 5.1 Parameters

| off | type | meaning | read at |
|---|---|---|---|
| 0x00 | s32 | rect x (virtual 400x240 screen pixels, origin top-left **INFERRED**) | `0x596c02`, `0x596c20` |
| 0x04 | s32 | rect y | `0x596c0d`, `0x596c40` |
| 0x08 | s32 | rect width | `0x596c1c` |
| 0x0c | s32 | rect height | `0x596c3c` |
| 0x10 | f32 | scalar start value v0 | `0x596cf0`, `0x596bd4` |
| 0x14 | f32 | scalar rate per FEP frame | `0x596be0`, `0x596bbe` |
| 0x18 | u8 | blend selector -> `0x5c02d0` (table `0x7ed8e0`: 0->1, 1->2, 2->3, 3->1) | `0x596e58` |
| 0x19 | u8 | 0 = full screen, else use the rect | `0x596bfb` |

FSH0 p10 (init and key 0, identical): rect (0, 0, 512, 448) unused because `+0x19` = 0,
v0 128.0, rate 0, blend selector 1 (-> index 2, additive). Refs: input 0 = command 3 (Colac),
input 1 = command 4 (Lmatrix, not read by the draw).

### 5.2 Work and the non-draw slots

```
init (slot 0):                                   # 0x596cb0
    A = alloc(0x10) from ctx+0x18 allocator; work[0] = A          # fail -> return 0
    A.value (+0) = p.v0; A.base (+4) = p.v0
    B = alloc(0xc); A.rect (+8) = B; B = 0 (six s16)
    return 1
update (slot 2, t = local time):                 # 0x596be0
    A.value = A.base + t * p.rate
    if p.use_rect (+0x19):
        B.x0 = p.x; B.y0 = p.y; B[2] = 0
        B.x1 = (p.x + p.w > 400 (0x9e84b0, unsigned)) ? screen_w : p.x + p.w     # u16
        if p.y + p.h > 240 (0x9e84b4, unsigned): B.y1 = screen_h
        else:                                    B.y1 = p.y + p.h
        B[5] = 0
    else:                                         # full screen
        B.x0 = 200 - screen_x_off ; B.y0 = 120 - screen_y_off
        B.x1 = screen_w ; B.y1 = screen_h ; B[2] = B[5] = 0
key (slot 1, new p, old p, old span s):          # 0x596bb0
    A.base += p.v0 - old.v0                      # (the s * old.rate terms cancel)
loop (slot 5, t_new, old p, t_old):              # 0x597160
    A.base += t_old * old.rate - t_new * p.rate ; then slot 2 (update)
release (slot 4): free A.rect, free A           # 0x596d40
```

`screen_w`/`screen_h` = `[0xaa6e28]`/`[0xaa6e2c]` (static init 400 / 240 from `0x9e87c8` /
`0x9e87d0`, `0x17ef0`, `0x17ea0`; rewritten at run time by `0x60ef75` for the PC aspect:
width = 2 x half-width); `screen_x_off`/`screen_y_off` = `[0xaa6e30]`/`[0xaa6e34]` (half-extents,
`0x60ef64`). `200`/`120` = `[0x9e87d4]`/`[0x9e87dc]`. So the full-screen rect always covers the
whole (possibly widened) screen. `A.value` (v0 + rate t) is computed but **nothing reads it**:
the draw uses only `A.rect` and input 0, and no command of FSH0 refers to the Flash command.

### 5.3 Draw (slot 3, `0x596da0`)

```
if elem_inst.byte[0x1b6] != 0: return            # 0x596db7 (INFERRED: element hidden)
vb = per-frame vertex memory (0x30 bytes = 4 x xyz f32)   (0x5e1b80)
state = 0x5e1960([0x9e87f8] & 0xf); frag = 0x5e19a0(); cmd = 0x5e1f20()
viewport/scissor reset to the full screen (0x5e30f0(state+0x168, cmd))
frag.depth (+0x40) = 0, dirty |= 0x20              # depth test off (INFERRED from render-state 1)
frag.blend = 0x5c02d0(p+0x18), dirty |= 1; 0x5e2300; flush 0x5e2ea0
c = input0 vec4 (Colac/Col output, 0..255)         # first ref's work (offset 0 if no ref)
c = c * elem_inst.colour (+0x40)                   # 0x63b0c0, per component
c = c * 2.0                                        # 0x63b100, [0x867e48]
clamp each component to [0, 255]; K = A<<24 | B<<16 | G<<8 | R    (bytes, truncated)
0x5e29a0(state+0xa4, cmd, K, 0)                    # the untextured "flash" combiner preset
x0 = B.x0 - 200, y0 = B.y0 - 120, x1 = B.x1 - 200, y1 = B.y1 - 120     # floats, z = 0
vb = (x0,y0,0) (x0,y1,0) (x1,y0,0) (x1,y1,0)        # one triangle strip, 0x596fb1..0x597074
shader [0xaaa958]; vertex format (vt+0x10)(0) = position only; vertex pointer 0x6007c0(vb)
world = identity (0x63d640 -> 0x601af0); 0x6018d0(.., 0) does nothing
0x5e33f0(.., prim 6, first 0, count 4); restore viewport (0x66f890); submit 0x5e0d30
```

Combiner preset `state+0xa4` (`0x5e2c80`): A 0, B 0x101, C 0x102, D 0, default K 0xff808080
(argument order of `0x6723f0`: A = edx, B = r8, C = r9, D = stack). By render-state.md 3's rules
(**INFERRED**, not re-read for this preset): untextured, no vertex colour -> vertex colour 128/255;
`Ambient.rgb = K.rgb / 128` (C low byte 2), `Ambient.a = K.a / 255` (C high byte 1); colorScale /
alphaScale = 1 (B bytes 1). That gives `rgb = K.rgb / 255`, `a = K.a / 255 x 128/255`. The
projection that maps the centred virtual-pixel coordinates to the screen is **not traced**
(INFERRED a 2D screen projection for the effect layer; for the full-screen case only "covers the
screen" matters).

**For a port**: a full-screen quad drawn after the scene, depth test off, blend index 2
(`src x src_alpha + dst`, additive), colour `rgb = clamp(2 x Col.rgb x E.rgb, 0, 255) / 255`,
alpha `clamp(2 x Col.a x E.a, 0, 255) / 255` (x 128/255 if the preset inference holds), where
Col = the part's Colac output this frame and E = the element colour (EAD RGBA / 128 x handle
alpha x owner alpha).

FSH0 part 10 (life 7 FEP frames, `ddd_fep_parts.py`): Colac init (128, 64, 128, 8), velocity
(0,0,0, 59.72), acceleration (0,0,0, -31.72); key 2: (128, 64, 128, 64), v (0,0,0, -25.43),
a (0,0,0, 5.052); key 7: alpha 0. With E = (1, 1, 1, 1.297): colour (255, 128, 255) = pink-purple,
alpha 2 x 1.297 x a(t): ~21 at frame 0, 166 (0.65) at frame 2, falling to ~0 by frame 7.
**A purple additive full-screen flash lasting 7 FEP frames (0.23 s) when Flowmotion starts.**

## 6. Open points

1. Abnormal exits from state 27 (hit, death, cutscene, area change): whether each passes
   `0x2a7b80` (aura stop) was not traced; only the normal exit is INFERRED.
2. FecFlash colour/alpha scaling in the combiner preset `+0xa4` (5.3) and the 2D projection:
   inferred from render-state.md, not re-read. Check: the `0x670db0` path for A = 0, B = 0x101,
   C = 0x102.
3. Element instance `+0x1b6` (Flash skips its draw when set): meaning not traced.
4. `0x2aa580` (aura suppression) trigger: what sets `+0x59c` bit 20 (INFERRED a hidden-player
   mode); irrelevant for KH1.
5. ~~Record table capacity~~ read (7.4): 3 x the largest event count of any motion, enough.
6. The FEP `A.value` scalar of Flash (+0x10/+0x14) has no reader in this element; another class
   might read it through refs in other files (not checked).
7. Writers of the owner model's colour `+0x50..+0x5c` after construction (7.1): none found in the
   model class code (`0x5e8000..0x5f4000`) nor next to a `+0x128` load in the player code; a
   setter elsewhere (player fade, cutscenes) is not ruled out. The port takes 1.0.
8. The handle colour filter `0x264f80` (7.1) and the element view / distance tests (7.4): not
   ported; both are 1 / pass in the normal case.

## 7. Why the port drew "huge, nearly white-hot blobs" (checked against the exe)

Symptom: in the KH1 port the ORA sprites covered Sora's limbs as big, saturated purple-white
blobs. DDD shows a purple glow **around** him. Everything was checked in this order; only 7.3
differs.

### 7.1 The owner's `+0x5c` (bit 8) and `+0x50` (bit 11): constant 1.0

- The owner passed to `0x5521e0` is the EAD controller's `+0x38` (`0x553c60`: `[ctrl+0x38] = r8`
  at `0x553d1f`, `[ctrl+0x40]` = the motion object), set by `0x5c9c80(motion obj, .., owner)`
  from `0x21bbb1` with owner = `[chara+0x68]`, the **model object** (alloc 0x138, constructor
  `0x5ea860` -> base `0x5ea960`; its `+0x100` is the skeleton that `0x550c80` reads, `0x5ea884`).
  The motion object (`0x5c78a0`, `+0x54` fps 60, `+0x5c` motion frame, `+0x88` the EAD ctrl,
  `0x5c9cd7`) is player `+0x130`; the model is the object before it (player `+0x128`, the one
  `0x2aa5f5` sets the hidden bit 0x40 of `+0x64` on).
- Model `+0x50..+0x5c` = colour vec4 (1, 1, 1, **1**), `+0x60` = packed 0xff808080, written by
  the base constructor (`0x5eaa07..0x5eaa26`). So bit 8's factor is the model's colour alpha
  (1.0), bit 11's the model colour (1, 1, 1, 1). No writer found afterwards (open point 7).
- `0x550c80` links the owner (`handle+0x70`) only when one of the handle's nodes carries the same
  `+0xac` id as the owner's first bone node; unlinked -> no multiply at all (`0x550f3d`).
- `0x550ef0` also multiplies the colour by the global colour filter `0x264f80(handle+0x64)`
  (table `0x9cf900`, 0x40-byte entries with a bit mask at `+0x10`, result clamped to 0..2): it
  returns null (no multiply) unless a filter with the handle's bit is active. Not a factor in
  normal play.

### 7.2 EAD fields of the 19 events (raw bytes re-read)

Tint `+0x14..+0x16` = 0 (no tint path, `0x5524a4`), ramp-in `+0x17` = 0, fade-in `+0x2f` = 0,
tint ramp-out `+0x32` = 0, change fade `+0x2a` = 6, end fade `+0x2e` = 6, flags 0x010e / 0x0106,
max 1. The handle colour is RGBA x 1/128 (`0x550c00`, `[0x7a94a0]`), copied unchanged to the
instance colour `+0xd0` -> `+0x40` (rgb), alpha `+0x4c` = stop fade x `+0xdc` x the element's
view factor (`0x5b49e2..0x5b49fd`). The port's tint (EAD RGBA / 128) is the same.

### 7.3 The cause: the depth push's sign

Every aura sprite has FecAnm_plus `+0x1d` = 1 (depth push on), `+0x1e` = 3 (view space) and a
stored vector **(0, 0, +0.3)**: ORA0 parts 0-2, ORA1 part 0, FOR0 parts 0-1, all 7 parts of the
Keyblade's W_010_SO_ORA0_0. Every other Flowmotion sprite stores a **negative** z (-0.5 / -0.7 /
-0.8: CAL0, DRL0, SHO0 part 6, BSS0, LOC1, MLC0, WAL0, FSH0, FOR0 parts 2-3), and FIR0 parts 0, 1,
12 store +0.5 (crate test `aura_sprites_push_away`).

DDD (draw-sprites 4.5): `0x5b39a0` negates y and z on read (`0x5b3a39..0x5b3a64`) and adds the
vector in view space; `0x5baa40` turns it into a clip-w shift scaled by the frame's z row length
(`w1 = w0 + (Q.w - w0) x zlen`, "negative = toward the camera", near clamp only for pushes
toward it). DDD's view looks down -z (projection m22 = -1, the test at `0x66fe00`), so a stored
-0.8 (negated +0.8) is tested **nearer** the camera and the aura's stored +0.3 (negated -0.3)
**farther**: 0.3 x 1.04..1.3 DDD units (31..39 KH1 units) behind the limb the sprite sits on.
The limbs and body hide the sprite's middle; only the part outside his silhouette shows: the
outline glow.

The port took `-|z|` for every sprite (and ribbon), i.e. always toward the camera: the aura's 42
additive sprites were tested 31 KH1 units in front of Sora and drawn whole over his limbs, their
saturated centres stacking (each centre adds up to (1.2, 0.7, 1.2) x texture alpha 1) into the
white-hot blobs. Fix: `sprite::depth_push` keeps the stored sign and scales by the frame's z
length (`0x5baa40`'s `zlen`, was the x length); `ribbon.rs` uses it too (its stored values are
negative: unchanged).

### 7.4 Sizes and the rest (equal in DDD and the port)

- ANM 0 of `p_fa_com0` / `w_010_so` (id 0xf00000ab, size 62, texture 0xf0000015 = resident
  `-21.dds`, cell u 0.754..0.996 x v 0.004..0.246: white RGB, a radial alpha glow, mean alpha
  44/255): one quad, vertices +-16384/32767 = +-0.5. ORA0 parts: Scl 6 / 5.5 / 5 % at local x 0 /
  -0.14 / -0.26; ORA1 7.5 %; Keyblade ORA0 9.5 / 6 / 5 / 5 / 5 / 5 / 7.5 %. Half-extent =
  0.5 x 62 x 0.1 x Scl x Sc: ORA0 on `L_momo` (EAD 1.04) 0.193 DDD = 19.3 KH1 units; ORA1 on the
  head (1.3) 0.30 DDD. The crate gives these (local 3x3 = Scl / 100, ANM size 62) and the port's
  `placement` mode 1 (Lmatrix byte 1 = 1: billboard, `0x5b15db..0x5b183f`, `diag(|frame rows|)
  x M x f x Vinv`) matches DDD's; Lmatrix byte 0 (= 1) is not read by `0x5b1510`.
- Colour: steady Colac (128, 128, 128, 128) (loop frames 3..4), x the element colour, `(q x C)
  >> 7`: port and DDD alike. The parts do not pile up: ORA0 keeps 3 live parts, ORA1 1, the
  Keyblade 7 over 150 FEP frames (looping body, looping parts).
- Record table (`0x552b40`): `[ctrl+0x60]` = {records, 3 x max(1, `0x5519c0`)}, `0x5519c0` = the
  largest event count of any motion of the EAD (`0x552c02`): Sora's `001` alone has 21, so all 18
  start.
- Element view / distance tests (`0x5b48e9..0x5b4b08`): ORA0's element bytes `+0x0a` = 2
  (bounds box +-3 around `+0x50`), `+0x0b` = 0 (distance test 124 / 240, table `0x9e8340` /
  `0x9e8358`), `+0x06` = 0: they hide or fade only far-off or off-screen effects.
- Element `+0x1b6`: read only by the Flash draw (5.3), not by Anm_plus.
