//! Dream Drop Distance's effect files (`effect/**/*.fep`): the container and the interpreter
//! that plays them, as DDD's exe does (Steam KH 2.8; every address an RVA from image base
//! 0x140000000). The format and the code it rests on: kh1-flowmotion's docs/fep/README.md.
//!
//! - `parse`: the whole file, read directly (file-relative offsets, no relocation), checked
//!   against DDD's command signature table (`sig`).
//! - `res`: resource bytes by index or id, and resident.frr.
//! - `run`: an element instance: its clock, spawn events, parts, keys, loops, stop and end.
//! - `cmd`: what each command class does (Pos/Rot/Scl/Col families, Lmatrix, generators,
//!   Gravity, Rnd*, Colorblink, the trail's Locus, the ribbon's Wave strategy); drawing classes keep their parameters and state for a drawer; the others
//!   are stubs (`cmd::class_status`).
//! - `rng`: the random source (DDD's own generator is not matched).
//!
//! ```no_run
//! # use std::sync::Arc;
//! let fep = Arc::new(fep::Fep::parse(std::fs::read("p_fa_ajc0.fep").unwrap()).unwrap());
//! let mut e = fep::Instance::start(fep, "P_FA_AJC_SHO0_0", 0.0).unwrap();
//! while !e.is_ended() {
//!     for p in e.parts() {
//!         for d in p.draws() {
//!             let _ = (p.index(), p.local(), p.colour(), d.class, d.resource, d.local_time);
//!         }
//!     }
//!     e.step(0.5);
//! }
//! ```

pub mod cmd;
pub mod parse;
pub mod res;
pub mod rng;
pub mod run;
pub mod sig;

pub use cmd::{class_status, ClassStatus, SCREEN};
pub use parse::{Error, Fep, ResKind, Resource, Result};
pub use res::Resident;
pub use rng::{Rng, XorShift};
pub use run::{DrawCmd, FlashDraw, Instance, LineDraw, LineVertex, Options, PartView, RibbonDraw, RibbonVertex, SpawnInfo, Sprite, TailDraw, TailPoint};
