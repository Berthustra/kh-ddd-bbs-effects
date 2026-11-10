# BBS's characters, Command Styles and effects (for kh1-combo's shift forms)

Read from the user's 1.5+2.5 install (Steam), 2026-11-07. Nothing here has run in game.

## Reading the files

- **Archives:** `ddd::archive_reader(install, "bbs", "<install>/KINGDOM HEARTS Birth by Sleep FINAL MIX.exe")` opens `Image/dt/bbs_*.hed` with the BBS exe's keys.
- **File names:** OpenKH's `D:/OpenKH/Apps/resources/bbs.txt` (64,108 names).
- **Extracted copies:** in `D:/KH-Extract/bbs`. `arc/pc/p0[1-3]*`, `arc/pc/xcs*`, `arc/pc/xfc*`, `arc/effect/*`, `arc/system/*` and `arc/pc_*` are there.
- **ARC packs:** `tools/bbs_arc.py` unpacks them. The header is `"ARC\0"`, `u16 version`, `u16 count`, then 8 zero bytes. Each entry is 0x20 bytes: `u32 0`, `u32 offset`, `u32 length`, `u32 0`, `char name[16]`.
- **Models:** PMO, the same as DDD's. `ddd::skeleton` reads them unchanged.
- **Motions:** PAM version 1. The `u16` at +0xE is 1, against DDD's 2. Each entry is `u32 offset` plus the name inline (12 bytes). `ddd::motions` reads both versions; `ddd::from_bytes(pmo, &[pams])` builds a set.
- **Effects:** FEP, the same as DDD's. The `fep` crate parses and starts them unchanged; tried on `w_17ve_00`, `w_17aq_00` and `d_cd_103`.
- **Effect events:** EAD, the same layout as DDD's.
- **`.ese`:** an effect's element list with a value per element, probably its sound.

## Characters

| | Ventus | Aqua | Terra |
|---|---|---|---|
| Model | `arc/pc/p01base/p01ex00.pmo` (53 bones) | `p02base/p02ex00.pmo` (54) | `p03base/p03ex00.pmo` (51) |
| Base motions | `p01ex_001.pam` | `p02ex_001.pam` | `p03ex_001.pam` |
| Per-command packs | `p01ex_NNN.arc` | `p02ex_NNN` | `p03ex_NNN` |
| All motions read | 167 | 159 | 153 |

- `p1X` (`p11`, `p12`, `p13`) are the same three characters in their armour.
- The bone names follow DDD's convention: `koshi`, `hara`, `mune`, `kubi`, `atama`, `L_/R_kata`, `ude1`, `ude2`, `te`, `yubiA`/`yubiB`, `buki`, `momo`, `sune`, `ashi1`/`ashi2`.
- The costume bones differ. `retarget::stand_in` maps them: the jacket to `L_jacA`/`B_jac`, the skirt to `susoA`.., the sleeves to `sode`/`sodeA`, otherwise the body bone they hang from.
- Test `pose::bbs_characters_retarget_onto_kh1`: every motion of all three at its start, middle and end is finite on KH1 Sora. Plotted poses (idle, attack, command) look right.

**Ventus's reverse grip** is in the motions: `R_buki`'s turn in the hand.
- The blade's +x in `R_te`'s frame:
  - In every bind pose: about (-1, 0, 0).
  - In Aqua's and Terra's motions it stays near -x: (-0.84, -0.31, -0.44) and (-0.79, -0.57, 0.21).
  - In Ventus's motions it flips to +x: (0.94, 0.23, 0.25) at idle, (0.98, 0.19, 0) at attack 003.
- The retarget carries `R_buki`'s turn relative to its bind onto KH1's grip bone 280. So KH1 Sora holds the Keyblade reversed when he plays Ventus's motions, and normally when he plays Aqua's or Terra's. Not seen in game yet.

## Command Styles

Each style's motions are in `arc/pc/xcsNN<who>.arc`, as `p0Xex_4NN.pam`. The suffixes:
- `ve` / `aq` / `te`: normal Ventus / Aqua / Terra.
- `vx` / `ax` / `tx`: the armoured versions (`p1X`).

Its effects are `arc/effect/pst_4NN/p_st_NN_<code>.fep`, with its `.ese`. Some packs also hold `.ecm` (style cameras) and extra models.

| NN | Who has its motions | Effect code | Style (ESTIMATE where marked) | Extras |
|---|---|---|---|---|
| 01 | te | fam | Critical Impact (Terra's first style) | cs01fin.ecm |
| 02 | aq | mag | Spellweaver (Aqua's first) | |
| 03 | ve | spr | Fever Pitch (Ventus's first) | |
| 04 | all | fib | Firestorm | |
| 05 | all | dmd | Diamond Dust | |
| 06 | all | thd | Thunderbolt | |
| 07 | ve | (no pst_407) | ESTIMATE: a Ventus-only level-2 style | |
| 08 | aq, te | (no pst_408) | ESTIMATE: Bladecharge (Terra and Aqua per KH wiki) | xcs08aq0 / te0.ecm |
| 09 | aq, ve | cyc | Cyclone | |
| 10 | te | blc | ESTIMATE: Terra-only | x01ex00.pmo, x04ex00.pmo |
| 11 | aq | (no pst_411) | ESTIMATE: Ghost Drive (Aqua-only) | three .ecm |
| 12 | all | rob | Rockbreaker by its code | |
| 13 | te | gsd | ESTIMATE: Terra's, rides a creature | x10ex00.pmo, x10ex_413.pam |
| 14 | ve | skr | ESTIMATE: Sky Climber by its code | x30/x31ex00.pmo with their own EAD (the extra Keyblades?) |
| 15 | all | dim | ESTIMATE | |
| 18, 19, 20 | (no xcs pack found) | wib, icb, rmr | Wingblade, ?, Rhythm Mixer (Final Mix) | motions not located yet |

**Where the names come from:**
- The style id, the characters and the effect codes are from the files.
- The names come from matching those against the KH wiki's list (Command Style page: who gets which style). They don't agree on every row: the wiki has Rockbreaker as Terra-only, but pack 12 has all three characters.
- The rows marked ESTIMATE need a look in game, or the BBS exe's style table near the `p_st_` strings at file offset 0x647678 / 0x65ad18.

**Effects per style:** for example, Diamond Dust's elements are `P_ST_DMD_HAN0/ICE0/ICE1/ICE2/MLC0/ORA1/ORA2`.
- `HAN` is the hand.
- `ORA` is the aura.
- `HIT` is a hit.
- `APP` is an appear effect.

The style effects are not in the characters' EADs (Ventus's `p01ex_000.ead` has only `FB_PC_ATR_0`). BBS starts them in code. We'll start them ourselves, per hit and per phase, as `fx_at` does.

## Other packs, not yet mapped

- `arc/pc/xfcNN`: finish commands.
- `xsl`: shotlocks.
- `xdl`: D-links.
- `xac`: attack commands.
- `pat_3NN` / `pac_2NN` / `pdl_6NN` effects: the same pattern for attack, action and D-link commands.
- `arc/effect/p0X_<who>`: each character's own (`p_at_com_v0.fep` with its EAD).
