# kh-fep: Dream Drop Distance and Birth by Sleep effects

By **Berthustra**. `kh-fep` stands for *Kingdom Hearts FEP*, the effect file format.

A reader and player for the visual effect files (`.fep`) of *Kingdom Hearts 3D: Dream Drop
Distance* and *Kingdom Hearts Birth by Sleep*, with a full write-up of the format: how an effect
is built, how the engine plays it frame by frame, how it is drawn, and how the characters' motion
events (`.ead`) start and stop effects on their bones.

As far as I know, this is the first public documentation of these formats. It was made while
porting DDD's Flowmotion and BBS's Command Styles into *Kingdom Hearts Final Mix*, where all of
it now draws in game (the Flowmotion & Shotlock for KH1 mod).

## What's here

| Folder | What it is |
|---|---|
| [`docs/fep/PUBLIC.md`](docs/fep/PUBLIC.md) | **Start here.** The format write-up: container, elements, parts, command classes, resources, drawing, timing, triggers. |
| [`docs/fep/`](docs/fep/README.md) | The detailed topic documents behind it (commands, generators, tails, sprites, models, render state, textures, aura, triggers, drawing them in KH1). |
| [`docs/bbs/`](docs/bbs) | Birth by Sleep: where its characters, Command Styles and effects live, its EAD event groups, its style elements. BBS uses the same FEP, EAD, PMO and PAM formats as DDD. |
| [`fep/`](fep/README.md) | A Rust library (`fep` crate) that parses FEP files and plays their elements as DDD's exe does. It does not draw; it hands a renderer each live part's matrices, colours and draw commands. |

## Games and builds

- DDD: `KINGDOM HEARTS Dream Drop Distance.exe` from **KH HD 2.8 Final Chapter Prologue** on
  Steam. Addresses in the docs are RVAs from image base `0x140000000` in that build.
- BBS: **KH HD 1.5+2.5 ReMIX** on Steam.

## No game data

Nothing from the games is included: no files, textures, sounds, code or disassembly. The docs
give offsets, field meanings and addresses so anyone can check a claim in their own copy. The
crate reads the files from your own install. One exception, for checking files: the 158-entry
command size table in `fep/src/sig.rs`, which every FEP file also carries in its header.

## Use the crate

```sh
cd fep
cargo test            # the tests on the game's files skip when they are not found
```

See [`fep/README.md`](fep/README.md) for the API. The tests and examples look for extracted DDD
files in `D:/KH-Extract/ddd/original`; change the paths at the top of
`fep/tests/flowmotion.rs` to yours.

## Credit

kh-fep is by **Berthustra**. If you use the docs or the code in your own tools, mods or write-ups,
please credit **Berthustra** and link this repository. The licences below require keeping the
copyright and [NOTICE](NOTICE) in any copy.

## Licence

MIT or Apache-2.0, at your option ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).
Kingdom Hearts is © Disney and Square Enix; this project is not affiliated with them.

## Thanks

- [OpenKH](https://github.com/OpenKH/OpenKh), for the BBS file name list.
- [KH2 Combat for KH1](https://github.com/PM2CRAZY/KH2-Combat-in-KH1), which the KH1 mod
  builds on.
