# FEP triggers: what starts each Flowmotion effect, and when

Scope: for each of the 19 Flowmotion elements (README 11.1), what starts it in DDD (an EAD
event or code), on which condition, where it is placed, what stops it; then which phase of our
mod (`src/flow.rs`) it belongs to. Placement, clocks and stop mechanics are in
[space-time.md](space-time.md) (cases A-D, sections 3 and 5); this file does not repeat them.
Exe: DDD Steam (KH 2.8), RVAs. **Labels:** unmarked = read in the exe or the files this session;
**INFERRED** = follows from what was read, a link not traced; **GUESS** = not read.

Tools: `tools/ddd_fep_triggers_ead.py --files p_fa_,p_ac_com0,w_010_so` (every event),
`tools/ddd_fep_triggers_summary.py` (that dump per element), `tools/ddd_fep_triggers_cmdrec.py`
(command -> patkdata record -> motion + effect group), `tools/ddd_fep_triggers_effgrp.py`
(groups passed to the motion setters), `tools/ddd_fep_triggers_starts.py` (string arguments of
the start-by-name calls), `tools/ddd_fep_triggers_writes.py` (every write of a struct field, e.g.
player `+0x698`).

---

## 0. The answer in one line

**Every one of the 19 elements is started by an EAD event, and none is started by code.** What
the code decides is *which* of a motion's events run: it passes an **effect group** with the
motion (section 2), and an event runs only when its group (`+0x12`) is negative, the passed group
is negative, or they are equal (`0x554070`, `0x55412b..0x554156`; space-time 2). So the
"condition" of every Flowmotion effect is (motion, group), and the group comes from the
Flowmotion command being executed (patkdata record `+4`), with one code override (the rail,
section 2.2).

## 1. EAD events per element (Sora `p_ex01_000.ead`, Keyblade `w_so01_001.ead`)

Times: EAD ticks (1/60 s of motion time); the motion frame is half (shown as "fr"). "grp" = the
event's effect group. Flags as space-time 2 (`0x1d` = follow + bone rotation + end-time stop
request + motion change = stop request; `0x1c` same but motion change = fade over `chg` ticks;
`0x19` = stay + bone rotation, stop request; `0x11` = stay, world axes; `0x0c` follow + bone
rotation, fade; `0x04` follow, world axes; `0x08` stay + bone rotation; `0x00` stay, world axes).
Offsets in DDD units (raw = KH1 units), rotations in degrees (X then Y then Z), scale x1 unless given.

| element | motion (what) | t0 (fr) | t1 (fr) | bone | offset / rot / scale / RGBA | grp | flags, chg | case |
|---|---|---|---|---|---|---|---|---|
| P_AC_COM_MSL0_0 | 015 air dash | 2 (1) | 16 (8) | Root | (0, 0.9, 0) | 110 | 0x1c, 8 | A |
| | 041, 042, 043 (cmd 10, not Flowmotion) | 0 | - | Root | (0, 0.95, 0) | 116 | 0x1d, 8 | A |
| P_AC_COM_MSS0_0 | 017 glide | 14 (7) | - | Root | (0, 0.8, 0.2); RGBA 153,89,153,128 | **200** | 0x1d, 8 | A |
| | 017 (plain) / 016 | 14 (7) | - | Root | 017: (0, 0.8, 0.2); 016: (0, 0.75, 0.12) rot (14,0,0) | 111 (016 also 200, tinted) | 0x1d, 8 | A |
| P_AC_COM_RNG0_0 | 017 glide | 14 (7) | - | mune | 0; RGBA 140,102,140,128 | **200** | 0x11, 8 | D |
| | 017 / 016 plain | 14 (7) | - | mune | 0 | 111 (016 also 200) | 0x11, 8 | D |
| | 321 wall kick | 4 (2) | - | mune | 0; RGBA 179,128,76,128 (orange) | 230 | 0x11, 8 | D |
| P_FA_COM_LOC1_0 | 017 glide | 14 (7) | - | hara | rot (-20, 120, 0) | **200** | 0x1d, 8 | A |
| | 016 | 14 (7) | - | hara | rot (20, -10, 0) | 200 | 0x1d, 8 | A |
| | 034 rail | 22 (11) | - | hara | rot (130, 0, -30) | 215 | 0x1d, 8 | A |
| | 035 pole | 0 | - | L_ashi2 | rot (90, 90, 0) | 201 | 0x1d, 8 | A |
| | 035 (second copy) | 0 | - | L_ashi2 | rot (90, 90, 0) | **255** | 0x1d, 8 | A |
| | 036 pole swing | 0 | - | L_ashi2 | rot (0, 90, 0) | 210 | 0x1d, 8 | A |
| P_FA_COM_LOC0_0 | 035 pole | 0 | - | L_ashi2 | rot (90, 90, 0) | 201 | 0x1d, 8 | A |
| | 036 pole swing | 0 | - | L_ashi2 | rot (0, 90, 0) | 210 | 0x1d, 8 | A |
| P_FA_COM_LOC2_0 | 035 pole | 0 | - | L_ashi2 | (1.0, -2.5, -1.0), scale 2 | 201 | 0x15, 8 | B |
| P_FA_COM_BSS0_0 | 034 rail | 22 (11) | - | Root | (0, 0, 0.5), fade-in 10 ticks | 215 | 0x1d, 8 | A |
| P_FA_BOS_BSS0_0 | 034 rail | 22 (11) | - | Root | (0, 0, 0.5), fade-in 10 ticks | **551** | 0x1d, 8 | A |
| P_FA_BOS_LOC1_0 | 034 rail | 22 (11) | - | hara | rot (130, 0, -30) | **551** | 0x1d, 8 | A |
| P_FA_COM_WAL0_0 | 030 wall run (wall on his left?) | 4 (2) | - | Root | (-0.7, 0.38, 0.38) rot (0, 90, 0) | 205 | 0x19, 8 | C |
| | 031 wall run (mirror) | 4 (2) | - | Root | (0.7, 0.38, 0.38) rot (0, -90, 0) | 205 | 0x19, 8 | C |
| | 033 top of the run-up | 4 (2) | - | Root | (0, 0.7, 0.7) rot (0, 180, 0) | 205 | 0x19, 8 | C |
| P_FA_AKD_DRL0_0 | 321 wall kick | 24 (12) | - | center | (0, -0.15, 0) rot (90, 0, 0), scale 0.8 | 230 | 0x0c, **14** | A |
| P_FA_AKD_CAL0_0 | 322 kick end | 6 (3) | - | mune | 0 | 230 | 0x04, 0 | B |
| P_FA_AKD_SHO0_0 | 322 kick end | 54 (27) | - | Root | 0 | 230 | 0x00, 0 | D |
| P_FA_AJC_CAL0_0 | 323 kick dive | 4 (2) | - | mune | 0 | 235 | 0x04, 0 | B |
| P_FA_AJC_SHO0_0 | 323 kick dive | 64 (32) | - | Root | (0, 0, 1.3) | 235 | 0x08, 0 | C |
| P_FA_ASK_MLC0_0 | 325 pole attack | 0 | 24 (12) | Root | (0, 1, 0) rot (180, 45, 5) scale (0.9, 0.8, 0.9); RGBA 102,102,179,128 | 245 | 0x1c, 8 | A |
| | 325 pole attack | 60 (30) | 114 (57) | Root | (0, 1, 0) rot (180, 45, 0) scale 1.25 | 245 | 0x1c, 8 | A |
| P_FA_COM_MLC0_0 | 327 throw | 12 (6) | - | Root | (0.1, 1.1, 0) rot (30, -120, -30) scale 1.2 | 255 | 0x1d, 8 | A |
| P_FA_COM_FIR0_0 | 327 throw | 32 (16) | - | Root | (-0.3, 1.45, 1.7) scale 1.5 | 255 | 0x19, 8 | C |
| W_010_SO_LOC0_0 | Keyblade EAD, 45 events, all `Z_00`, group -1 (every group), flags 0x1c, chg 8 | | | Z_00 | 0 | -1 | 0x1c, 8 | A |

Keyblade trail events in the motions our mod plays (t0-t1 in ticks; frames in brackets):
321: 10 (5), no end; 322: 42-50 (21-25); 323: 52-68 (26-34); 325: 2-24 (1-12) and 62-110 (31-55).
None in 015, 017, 030-037, 327. (The other 38 are in ground combos, 319/320/324/326 etc.)

Every Flowmotion event has fade-in 0 (except the two rail BSS0, 10 ticks), end fade 0 (ASK MLC0:
14, unused because bit 4 makes its end a stop request), tint off, level 0. `max` (`+0x33`) is 1
for the glide, rail, pole and 041-043 events and 0 for the rest. The start counter (event state
`+0x10`) is only incremented in `0x5545a0` (`0x55477a`, `0x55481c`) and zeroed when the motion's
event states are built (`0x554070`): **max 1 = once per motion play**, so a looping 034/035/017
does not restart them each lap (the rail's t0 = frame 11 lies inside its 10-21 loop).

**No element of the 19 lacks an EAD start** (`ddd_fep_triggers_summary.py`: "FM not started:
set()"). The table in kh1-renderer.md 6.1 agrees.

## 2. Which code passes which group (the conditions)

### 2.1 The command records

The player's command id selects a 20-byte record of `game/bin/patkdata.bin` (switch `0x2a8b00`,
record `0x2a9000`); record `+0` = motion, `+4` = effect group. Entering the Flowmotion state
(27, `0x2c3cd0`) stores the command (`+0x84c` from `+0x6b2`), looks up its record (`0x2c3dfe`,
`0x2c3e0d`) and keeps it at player `+0x818`; every later motion change through `0x2a6070` takes
its group from that record (`0x2a61a7`: `movsx r14d, word [rec+4]`) unless player `+0x698` holds
an override (> 0, used once and cleared, `0x2a61c7..0x2a61d7`). `0x2a5dd0` takes the group in r9d
(`0x2a5df5`). (`0x2a6070`'s own r9d is not the group: it is a record byte, `0x2a61b1..0x2a61c3`.)

`ddd_fep_triggers_cmdrec.py`, Flowmotion commands (names from the exe's Lua constant table):

| cmd | Lua name | record | motion | group | effects that then run (section 1) |
|---|---|---|---|---|---|
| 3 | | 30 | 008 (roll) | 105 | none (no events in 008) |
| 6 | | 33 | 015 (air dash) | 110 | MSL0 |
| 7 | | 34 | 016 | 111 | MSS0 + RNG0 (untinted) |
| 8 | | 35 | 017 | 111 | MSS0 + RNG0 (untinted), no LOC1 |
| 10 | | 37 | 041 (040-043) | 116 | MSL0 (one per motion) |
| 28 | WallKick | 49 | 033 | 205 | WAL0 |
| 29 | PoleTurn | 50 | 035 | 201 | LOC0 + LOC1 + LOC2 on the left toes |
| 30 | PoleSwing | 51 | 036 | 210 | LOC0 + LOC1 |
| 31 | RailSlide | 52 | 034 | 215 (or 551, 2.2) | COM BSS0 + COM LOC1 (or BOS BSS0 + BOS LOC1) |
| 32 | SuperSlide | 53 | 017 | 200 | MSS0 + RNG0 (purple tint) + LOC1 at `hara` |
| 34 | | 54 | 321 (and 322) | 230 | RNG0 (orange), DRL0; 322: AKD CAL0, AKD SHO0 |
| 35 | | 55 | 325 | 245 | ASK MLC0 x2 |
| 36 | SideThrow | 56 | 327 | 255 | COM MLC0, COM FIR0 (and 035's second LOC1) |
| 39 | | 59 | 323 | 235 | AJC CAL0, AJC SHO0 |

The wall run (030/031) is set through `0x2a5dd0` with r9d = 205 (`0x2c28ee`, `0x2c3544`,
`0x2c3f15`, `0x2c4030`: `mov r9d, 0xcd`), so WAL0 runs there too. The launch `037` is set with
group 220 (`0x2c42b2`); no event has group 220 and `037` has no events anyway: **the launch starts
nothing**. The glide `017` inside state 27 is set at `0x2c408f` with the state's record (so its
group is that command's) unless the previous command was 29/30 (`0x2c4060..0x2c4085`).

**INFERRED:** the Flowmotion glide (Square after the super jump) is command 32 `SuperSlide`
(its record is motion 017, it is one of the state-27 commands 28-39 by name), so it runs the
**purple** set (group 200: MSS0 tinted 153,89,153, RNG0 tinted 140,102,140, and COM LOC1 at
`hara`). Commands 7/8 (016/017, group 111) are the same motion for another action (a deck
command, **GUESS**) and run the untinted MSS0/RNG0 without LOC1. Not checked against a recording.

**INFERRED:** 035's second LOC1 (group 255, SideThrow's) means 035 is also played under the
SideThrow record, i.e. the spin round a big foe before the throw (recording 7) runs **only that
one LOC1** (left toes, rot (90, 90, 0)), not the pole's LOC0 + LOC1 + LOC2. If the foe spin
runs under PoleTurn instead, it gets the pole's three. Not traced (open 1).

### 2.2 The one code override: the rail's boss variant

Both rail entries (`0x2c1359..0x2c138c` and `0x2c47c7..0x2c47ee`) start `034` (action 60) with
`+0x698 = ([0xa97fdc] & 0x4000000) ? 0x227 (551) : 0xd7 (215)` and clear it after. Bit 26 of
`[0xa97fdc]` is set only by the Lua binding **`SetBattleFlagLightEffectFFA`** (`0x28d7e0`, table
entry `0x7ad6b8`: `flags = flags & ~bit26 | arg << 26`). So a map/battle script that calls it
switches the rail to `p_fa_bos_0` (BOS BSS0 + BOS LOC1, group 551); otherwise the rail runs
`p_fa_com0` BSS0 + LOC1 (group 215). Nothing else reads that bit near Flowmotion. For KH1: use
215 (COM) everywhere; BOS is a scripted boss-arena variant.

### 2.3 Code that starts effects by name: none of the 19

- Name strings in the exe: of the Flowmotion files only `p_fa_com0` (`0x7887c8`) with
  `P_FA_COM_RNG0_0` (`0x788798`) and `P_FA_COM_SBT0_0` (`0x7887a8`), both through `0x11c950 ->
  0x5567d0` (`0x11c9b3`). Neither is one of the 19. `p_ac_com0` / `p_fa_asd0` appear only in the
  table at `0x7b0ac0` (`{name, u16 element, u16 kind}`: `p_ac_com0` elements 2, 3 = WRP0/WRP1,
  the warp), not Flowmotion. `W_170_SO_LOC0_0` (`0x7b7a80`, `0x32b553`...) is another weapon's
  trail, not Sora's `W_010`.
- `game/bin/hiteffect.bin` names `p_fa_com0` element 15 = `P_FA_COM_SHT0_0` (a hit effect), not
  one of the 19. No other data file outside the EADs/ESEs names a Flowmotion element.
- No exe constant equals a Flowmotion element id (`0x24b8xxxx`, `0x2000x400`, `0x389c0x00`).
- `"%s.fep"` (`0x786748`, used at `0xfc305`): the file name is the actor's own (`[obj+0x364]`),
  the element from `0x6c0c00(+0x4c)`; an actor-config effect, not Flowmotion.
- `0x5521e0` has one caller (`0x552add` in `0x552990`); `0x552990` is reached from the per-frame
  test `0x5545a0` and from `0x5528b0` / `0x553ac0` (start a motion's events with no end time at
  once, by motion name `0x553a80`). The player calls `0x553a80` only with motion `"001"` (the
  aura FOR*/ORA* events, `0x2a4daf`, `0x2bcbe3`...); other callers are other actors (`"030"`,
  `"010"`). `0x5ac210` has two callers (`0x5acd54` in the start chain, `0x5b7132` spawn events).

So the mod needs no code trigger beyond "play the motion with the right group".

## 3. Start, follow, duration, stop (per element, from section 1 + space-time 3 and 5)

Body lengths in FEP frames (29.97 a second; README 11.1). "Stop on change" = what the next motion
does to it (space-time 5.2): mode 1 = stop request at once (a looping body with stop mode 3 stops
looping and ends when its parts have died); mode 0 = alpha fade over `chg` ticks then kill;
non-looping bodies are left to play out.

| element | loops / len | follows? | stop |
|---|---|---|---|
| AC MSL0 (015) | yes / 30 | yes, Root + 0.9 up, full rotation | stop request at t1 = frame 8 (finishes its parts); earlier change: 8-tick fade |
| AC MSS0 (017) | yes / 30 | yes, Root (0, 0.8, 0.2), full rotation | stop request when 017 ends (mode 1) |
| AC RNG0 (017, 321) | no / 30 | no: placed once at `mune`'s position, world axes | plays its 30 frames (1.0 s) |
| COM LOC1 (017 `hara`, 034 `hara`, 035 `L_ashi2`) | yes / 40 | yes, full rotation | stop request when the motion ends |
| COM LOC0 (035) | yes / 40 | yes, `L_ashi2`, full rotation | stop request when 035 ends |
| COM LOC2 (035) | yes / 40 | position of `L_ashi2` + offset rotated by it, world axes, scale 2 | stop request when 035 ends |
| COM BSS0 (034) | yes / 30 | yes, Root + 0.5 forward, full rotation; 10-tick fade-in | stop request when 034 ends |
| BOS BSS0 / LOC1 (034, scripted) | yes / 30, 40 | as COM | as COM |
| COM WAL0 (030/031/033) | no / 20 | no: placed once from Root at frame 2, Root's rotation | plays out (0.67 s) even after the motion changes |
| AKD DRL0 (321) | yes / 30 | yes, `center` | on 321 -> 322: alpha fade over 14 ticks, then kill |
| AKD CAL0 (322), AJC CAL0 (323) | no / 30, 35 | position of `mune`, world axes | plays out |
| AKD SHO0 (322), AJC SHO0 (323) | no / 45 | no (322 world axes at Root; 323 Root rotation, 1.3 ahead) | plays out |
| ASK MLC0 (325) | yes / 30 | yes, Root (0, 1, 0) | stop request at t1 (frames 12 and 57) |
| COM MLC0 (327) | no / 20 | yes, Root, full rotation | plays out |
| COM FIR0 (327) | no / 20 | no: placed once at frame 16, Root's rotation | plays out |
| W_010 LOC0 | yes / 6 | yes, `Z_00` (the tip) | stop request at t1; on an earlier change 8-tick fade; 321's has no t1: fades 8 ticks when 321 ends |

No code sets any parameter of these effects: colour, scale, tint and fades are the EAD's. The
only code input is the group (section 2).

A trigger whose t0 lies before the frame a motion starts at fires at once, run ahead by the time
missed (`0x55479d..0x5547ac`: fires when motion time >= t0 and the event has not fired;
space-time 3.1 step 5). So a glide begun at 017 frame 13 starts its frame-7 effects at once,
3 FEP frames in (INFERRED from the branch; the late-start path was read in space-time).

## 4. Our phases (`src/flow.rs` `Phase`, the motion each requests at `flow.rs:1490..1507`)

| phase | DDD motion, group | effects to start (motion frame) |
|---|---|---|
| `Free`, `Pending` | - | none |
| `Dash` | 015, 110 | AC MSL0 at 1, stop request at 8; Keyblade none |
| `Roll`, `RunUp` | 008, 032 | none |
| `WallRun` | 030 / 031, 205 | COM WAL0 at 2 (the mirrored event for 030 vs 031) |
| `Top` | 033, 205 | COM WAL0 at 2, offset (0, 0.7, 0.7) rot 180 |
| `Drop` | 006 | none |
| `Launch` | 037 (018 after a glide) | none |
| `Glide` | 017, 200 (SuperSlide, INFERRED) | AC MSS0 (purple), AC RNG0 (purple), COM LOC1 at `hara`, all at 7 (i.e. at once, 3 FEP frames late, since the glide starts at 13); MSS0/LOC1 stop request on leaving |
| `Pole` | 035, 201 | COM LOC0 + LOC1 + LOC2 on `L_ashi2` at 0; stop request on leaving (launch or attack) |
| `FoeSpin` | 035, 255 or 201 (open 1) | 255: COM LOC1 on `L_ashi2` only; 201: as `Pole` |
| `Rail` | 034, 215 | COM BSS0 (Root, 0.5 ahead, 10-tick fade-in) + COM LOC1 (`hara`) at 11; stop request on leaving |
| `KickDive` | 323, 235 | AJC CAL0 at 2, AJC SHO0 at 32 (placed once); Keyblade LOC0 26-34 |
| `WallKick` (321 part) | 321, 230 | AC RNG0 (orange) at 2, AKD DRL0 at 12 (fades 14 ticks into 322); Keyblade LOC0 from 5 (fades 8 ticks into 322) |
| `WallKick` (322 part) | 322, 230 | AKD CAL0 at 3, AKD SHO0 at 27; Keyblade LOC0 21-25 |
| `PoleAttack` | 325, 245 | ASK MLC0 0-12 (blue tint, scale (0.9, 0.8, 0.9)) and 30-57 (scale 1.25); Keyblade LOC0 1-12, 31-55 |
| `Throw` | 327, 255 | COM MLC0 at 6, COM FIR0 at 16 |

Not reached by our phases: COM LOC0/LOC1 from 036 (PoleSwing, group 210: we have no 036),
the MSL0 of 041-043 (cmd 10), the 016 and group-111 sets, the BOS rail pair.

Implementation rule for the mod: keep, per phase, the DDD motion and group; run the EAD events of
that motion whose group is negative or equal (section 0), with the clock rules of space-time 4
(t0/t1 in motion ticks = 2 x the motion frame as the phase computes it), and on a phase change
apply each running effect's motion-change mode (space-time 5.2). Since the phases already request
the motion and frame each tick (`request(...)` calls), the trigger check can sit there.

## 5. Open points

1. The foe spin's group: 255 (SideThrow, only one LOC1) or 201 (PoleTurn, three). Find which
   command `+0x84c` holds when state 27 plays 035 at a foe (trace `0x2c3cd0`'s callers for the
   foe-catch path, or record `+0x84c` in DDD).
2. The Flowmotion glide's command (32 SuperSlide, group 200, purple) versus 8 (group 111,
   untinted, no LOC1): INFERRED from names only; a recording of `+0x84c` during the glide would
   settle it.
3. (Settled: max 1 = once per motion play, section 1.) For max-0 events in a looping motion the
   per-lap "fired" bit resets on a wrap, so they would refire each lap; none of ours is in a
   looping motion except the Keyblade's 321 event (t0 5, 321 loops 16-21: no refire).
4. Who sets bit 26 of `[0xa97fdc]` in which scripts (`SetBattleFlagLightEffectFFA`): not needed
   for KH1 (use COM).
5. 016's group-200 events (the glide's set with other offsets): which action plays 016 under
   SuperSlide, if any, was not found.
