# The KH1 side: drawing DDD's Flowmotion effects in KH1 (design, no code)

Design of the part of kh1-flowmotion that draws the FEP effects (README.md in this folder) inside
KINGDOM HEARTS FINAL MIX (Steam), with or without KH2 Combat for KH1. It is built from KH2
Combat's own effect renderer (`crates/kh1-hook/src/win/fx12.rs`, MIT OR Apache-2.0).

Nothing here was run in the game. Every KH1 address is an RVA in the Steam exe (image base
0x140000000, PE stamp 1721645006 = 0x669E37CE). **Labels:** unmarked = read from the exe or the
files, with the check named; **INFERRED** = follows from checked facts; **GUESS** / **ESTIMATE** =
not read, to be confirmed; "KH2 Combat says" = taken from KH2 Combat's docs or code, not re-read
here.

Contents: 1 what we take from KH2 Combat - 2 Steam addresses - 3 coexistence with KH2 Combat -
4 pipeline states and blend modes - 5 textures - 6 attachment (bones, units, axes) - 7 timing -
8 module plan and data flow - 9 risks - 10 open points - 11 tools.

---

## 1. What we take from KH2 Combat

Read: `crates/kh1-hook/src/win/fx12.rs` (921 lines), `kh2fx.rs` (992), `hook.rs`, `addr.rs`,
`steam_map.in`, `mod.rs` (start-up), `model.rs` (its replay hook), `leftblade.rs` (weapon bones);
`docs/kh1-render.md`, `docs/steam-epic-addresses.md`, `docs/kh1-weapon-second.md`,
`docs/kh1-clash.md`; `crates/kh2-fx/src/output.rs` (the `Frame` that feeds fx12).

### 1.1 How fx12 draws (as read in fx12.rs)

- It records D3D12 commands **into KH1's own command list**, on the game thread, right after
  KH1's world-model pass (task 160000). The world is drawn by then; KH1's own effects and the HUD
  come after.
- It has **no descriptor heap**. The root signature is 40 root constants (b0) and one root SRV
  (t0). The constants are View (16), P (16), KH1's clip scale and x/y offsets (3), and per batch:
  view-z depth offset, texel offset, width, height, flags (5).
- All textures sit in **one raw buffer** of RGBA8 texels (a `ByteAddressBuffer`, upload heap).
  The pixel shader fetches four texels and filters bilinearly by hand, with wrap.
- Vertices are 36 bytes `{pos f32x3 (KH1 world), uv f32x2, colour f32x4}`, triangle lists. They go
  into a 24 MB ring in an upload heap (4 x 6 MB, written round-robin, 256-byte aligned).
- Shaders are HLSL compiled at run time (`D3DCompile`, d3dcompiler_47/46/43). The root signature
  is serialised with `d3d12.dll`'s `D3D12SerializeRootSignature`.
- The clip transform is KH1's own: `c = (p,1)·View·P`, `clip = (c.x·s + c.w·ox, -(c.y·s +
  c.w·oy), z·c.w, c.w)`, with `s, ox, oy` from KH1's constant-buffer shadow.
- Per draw: get KH1's current list (`0xFD200(R)`); if it is a **direct** list set the render
  target (RTV + DSV); if it is a **bundle**, set nothing (a bundle inherits its targets); set root
  signature, topology, vertex buffer, constants, SRV; draw the batches; then **hand KH1 a fresh
  list**: `0xF7160(R, 0)` (close and queue) and `0x1091C0(R, 1, 0)` (open a new one with KH1's
  root signature and heaps, all KH1 state marked dirty). KH1 does exactly this pair itself when its
  vertex ring overflows mid-scene (section 2).
- Any failure logs once and turns the renderer off for the session. It touches nothing when no
  effect is drawing.

### 1.2 How kh2fx feeds it

- A background thread loads the effect packages and hands **all textures at once** to
  `fx12::set_textures(Vec<(w, h, rgba)>)`; fx12 builds the texture buffer on its first draw.
- Effects **step once per KH1 game frame** from KH2 Combat's controller tick (they stop when the
  game stops), and are **built and drawn** in the world-pass hook with the camera read there.
- It does not draw while KH1's pause machine (`0x232DFB8` Steam) is non-zero, nor when the pause
  menu's copy of Sora was posed in the last 150 ms (the pause menu runs a second world pass with
  its own camera).
- Bone anchors are world matrices supplied per tick through a callback (`bone(code)`).

### 1.3 How KH2 Combat hooks, and its address table

- **Its own inline hook** (`hook.rs`), not a library. `install(target, expect, ours, ...)`
  compares the first `expect.len()` bytes with KH1's own and **refuses to hook** if they differ
  ("unexpected bytes ..., not hooked"). It writes `E9 rel32` to a stub near the exe, NOPs the rest
  of the moved bytes, and builds a trampoline from the expected bytes. `redirect_call` repoints a
  `call rel32` site the same way.
- All hooks go in **inside its `DllMain`**, one after the other (`mod.rs`: `takeover::install()`
  ... `kh2fx::start()`). The world pass is hooked in `kh2fx::start()`, and only when its
  `kh2_effects` setting is not `off` (or `fx_test` is on).
- Addresses are written as **Epic** RVAs (`Rva(0x...)`); on Steam they go through
  `steam_map.in` (401 lines, Epic -> Steam, matched function by function; build picked by PE
  stamp plus three signature checks, `addr.rs`). We are Steam only (`game.rs` refuses other
  builds), so we write Steam RVAs directly, each checked by bytes at start-up.

### 1.4 License

fx12.rs and the egs reader are MIT OR Apache-2.0, "Copyright 2026 kh2-combat contributors". The
adapted file keeps that line and says what changed (section 8.4); `NOTICE` gets an entry in the
form it already has for `egs.rs` and `retarget.rs`.

---

## 2. Steam addresses (all checked by `tools/ddd_fep_kh1render_verify.py verify`: 27/27 pass)

### 2.1 Functions and globals fx12 / kh2fx use

| what | Epic (KH2 Combat) | **Steam** | how it was verified on Steam |
|---|---|---|---|
| renderer singleton R | `0x22BB580` | **`0x22BAA20`** | `0xFD820` (Epic `0xFD9C0`, the getter) is a thread-safe local static: `lea rax, [rip+...]` -> `0x22BAA20` at `0xFD852` |
| device `R+0x2E0` | same offset | `R+0x2E0` | `0x105730` does `lea r14, [r15+0x2E0]; mov rdx, r14; call 0xEFF30` (r15 = R: `[r15+0x3F0]` HWND, `R+0x4D0/0x4D8` output/adapter, as Epic); `0xEFF30` keeps rdx in r12 and passes `r9 = r12` (ppDevice) to `D3D12CreateDevice` (d3d12.dll ordinal 101, import thunk `0x35B6EE`) at `0xF011B` |
| View (as drawn) | `0x2E9DE60` | **`0x2E9D480`** | written by `0x1ED720` (four `movaps` to `0x2E9D480..0x2E9D4B0`); read by the world pass at `0x1ED058` |
| Screen = P·S | `0x2E9DFE0` | `0x2E9D600` | written by `0x1ED720` |
| GPU camera copy | `0x1EB5C0` | `0x1ED720` | prologue `48 89 5C 24 08 48 89 6C 24 10 48 89 74 24 18 57 48 83 EC 20` = the one kh1-render.md gives; called by the 140000 task below |
| camera task (140000) | `0x1DEDE0` | `0x1E0F40` | registered with `mov ecx, 0x222E0; call 0x284670` at `0x1E137D`; calls `0x1ED720(rcx = [0x233FEA0], rdx = [0x233FEA8], r8, r9, [rsp+0x20] = rdx+0x100)` |
| projection pointer | `0x2340EE0` | **`0x233FEA8`** | the `rdx` (projection struct) of that call; 8 refs (writers in event code) |
| view pointer | `0x2340ED8` | `0x233FEA0` | the `rcx` of that call; 12 refs |
| field P copy | `0x2341320` | **`0x2340A00`** | passed with the View copy `0x2340980` (= P - 0x80, as Epic) to the shader-param setter `0x107400(0x1000, ...)` in `0x199FE0`; 3 refs |
| command-list getter | `0xFD3A0` | **`0xFD200`** | returns cached `R+0x5990`, else opens from the wrapper `R+0x5988` (a bundle via `0xFE540`, Epic `0xFE6E0`, when the wrapper says so) |
| close and queue | `0xF7300` | **`0xF7160`** | the ring-overflow sequence below; reads `R+0x59A0`, `R+0x5988`, pushes the wrapper, takes the next pool buffer (`R+0xC90`), clears `R+0x5988/0x5990` |
| reopen | `0x109360` | **`0x1091C0`** | same sequence; sets `R+0x5990 = 0`, `R+0x5988 = pool buffer`, walks the frame buffers by `R+0x6C` |
| KH1's own close + reopen | `0xFBBA0` | `0xFBA00` | `inc [rdi+0xE378]; xor edx,edx; call 0xF7160; xor r8d,r8d; mov dl,1; mov rcx,rdi; call 0x1091C0` at `0xFBACA..0xFBADF`; the same pair at `0xFBDD8 / 0xFBDE5` |
| world-model pass | `0x1EADD0` | **`0x1ECF30`** | first 15 bytes `48 8B C4 48 89 58 08 48 89 70 10 48 89 78 18` = KH2 Combat's `WORLD_PASS_PROLOGUE`, then `55 41 54 41 55 41 56 41 57`; one caller |
| its task stub (160000) | `0x1DEE40` | `0x1E0FA0` | `sub rsp,0x28; cmp dword [rip+..], 0; je; call 0x1ECF30`; registered with `mov ecx, 0x27100; call 0x284670` at `0x1E1393` |
| its call site | `0x1DEE4D` | **`0x1E0FAD`** | bytes `E8 7E BF 00 00` (same as Epic: same relative distance) |
| task register | `0x282500` | `0x284670` | both registrations above |
| pause machine state | `0x232E9B4` | **`0x232DFB8`** | 18 refs from `0x17EF90..` (the pause code), e.g. `cmp [state], 6`, `mov [state], 3` |
| frame step (f32) | `0x23405DC` | **`0x233FBDC`** | 426 float reads; KH2 Combat says 1.0 at 60 fps, 2.0 at 30 |
| interpolated replay | `0x102740` | `0x1025A0` | KH2 Combat's `REPLAY_PROLOGUE` `48 8B C4 48 89 58 18 48 89 70 20 57`; sets `R+0x59A0 = 1`, `R+0x58F0 = 2`; called from the frame end `0x109D6E` and the pacing thread `0x10A453` |
| point along a bone | `0x29C990` | `0x29EB20` | `(actor rcx, bone code edx, t xmm2, out r9)` read from its first instructions; 100 callers |

### 2.2 Struct offsets fx12 reads (inside R and the frame buffers)

| field | used for | Steam check |
|---|---|---|
| `R+0x6C` bound frame buffer index | `target()` | read by reopen `0x10925B` to pick the frame buffer: 0 -> `R+0x540`, 5 -> `R+0x660`, else `R+0x688 + i·0x120` (`lea rcx,[rax+rax*8]; shl rcx,5`) |
| fb `+0x08/+0x0C` width/height | log, scissor | reopen reads `R+0x548` (fb 0 + 8) |
| fb `+0x20` RtvId, `+0x50` RTV increment | RTV handle | reopen reads `+0x20` (`R+0x560/0x680/0x6A8`); `0xF11F0` computes `heap start + [fb+0x50]·[fb+0x20]` and indexes the colour table `fb+0x108` by `RtvId·0x48` |
| fb `+0x24, +0x38, +0x40, +0x58, +0xA0, +0xA4` | DSV id, heap starts, DSV increment, MSAA count/quality | **not re-read on Steam: INFERRED** same as Epic (same source compiled the same day; every neighbouring field above matched). fx12's first-draw log prints them; check it |
| `R+0x59E0` cbConstantVS shadow (0x90 B) | clip constants | `0xF31B0` takes `lea r14, [rcx+0x59E0]` and compares 0x90 bytes with its copy at `+0x90` before uploading |
| `R+0x59F8 / +0x59FC` xyOffset.z/w | ox, oy | written by `0xF46BE` (`movss [rdi+0x59F8], xmm3` / `[rdi+0x59FC], xmm0`) |
| `R+0x5A10` SafeScale | s | written by `0xF38EC` (`movss [rdi+0x5A10], xmm0`) |
| `R+0x15C..+0x16B` viewport x, y, w, h | viewport for direct lists (new, 3.3) | read/written by the state flush (`0xF3ACB`, `0xF3AF0`, `0xF3B2F`) and setters (`0xFB6BF`, `0x105FF5`, `0x108D00`) |
| `R+0xC90` command-buffer pool | budget guard (new, 3.3) | `0xFD150`: buffer = pool `+0x38 + (slot·32 + used[slot])·0x88`, slot `pool+0x4AF8` (= `R+0x5788`), used `pool+0x4438+slot·4` (= `R+0x50C8`); close increments used at `0xF7289` |

Projection struct layout (`+0x80` = P) and the frame-buffer formats (colour B8G8R8A8_UNORM = 87,
depth D32_FLOAT_S8X24_UINT = 20) are KH2 Combat's reading (kh1-render.md 1.1, 2.2), not re-read.

---

## 3. Coexistence with KH2 Combat

### 3.1 Two hooks on the world pass

- **KH2 Combat** puts `E9 rel32` + 10 NOPs over the 15-byte prologue of `0x1ECF30` (when its
  `kh2_effects` is not off). Its detour calls the original, then draws its KH2 player model
  (`model::draw`, currently switched off in its code) and its effects.
- **Ours** uses retour 0.3.1 `RawDetour` (`src/hook.rs`, `src/pose.rs`). Read in retour's source
  (`arch/x86/trampoline/mod.rs`): a leading `jmp rel32` is an unconditional jump, so the builder
  stops there, emits an **absolute jump to the same target** in the trampoline, and patches only
  5 bytes. So **retour over KH2 Combat chains correctly**: KH1 -> our detour -> our trampoline ->
  KH2 Combat's detour -> its trampoline (KH1's real prologue) -> KH1's pass. Our physics-step hook
  already runs over KH2 Combat's this way and works in game (kh1-notes.md, every test "KH2 Combat
  on").
- **The other way round fails silently for KH2 Combat**: if ours goes in first, KH2 Combat's
  byte check sees our `E9` and refuses ("not hooked"): its effects (and its KH2 model, if turned
  on) would never draw. Our `hook::wait_for_kh2_combat` exists for exactly this.

### 3.2 Recommended hook point: the call site `0x1E0FAD`, not the prologue

Patch the 5-byte `call 0x1ECF30` at `0x1E0FAD` (in the 160000 task stub) to call a stub of ours,
which calls `0x1ECF30` and then draws. This:

- does not touch `0x1ECF30`'s bytes, so KH2 Combat's prologue hook installs whether it comes
  before or after us (it checks only the prologue);
- still runs after KH2 Combat's draws, because our call to `0x1ECF30` goes through its hook;
- runs only when the stub would call the pass (the stub's own flag test stays KH1's).

Check before patching: the 5 bytes are `E8 7E BF 00 00` and resolve to the pass (KH2 Combat's
`redirect_call` does the same check; none of its sites is this one). Needs a stub within 2 GB
(VirtualAlloc near the exe, as KH2 Combat's `alloc_near`), `VirtualProtect`,
`FlushInstructionCache`.

**Fallback** (bytes differ: another mod patched the site): a `RawDetour` on `0x1ECF30`, installed
after KH2 Combat's. Our existing wait ends when KH2 Combat's physics hook appears, plus 500 ms.
That is not a guarantee that its `DllMain` finished. Better: once `kh1-kh2.dll` is seen, call
`LoadLibraryW("kh1-kh2.dll")` and `FreeLibrary` (INFERRED: LoadLibrary takes the loader lock,
which is held while its `DllMain` runs, so it returns only after all its hooks are in).

### 3.3 Two renderers closing and reopening KH1's list each frame

- **KH1 does it itself.** The pair close `0xF7160(R,0)` + reopen `0x1091C0(R,1,0)` is KH1's own
  answer to a full vertex ring (`0xFBACA`, `0xFBDCD`), so several per frame are normal. KH2 Combat
  alone already does it up to twice a frame (its KH2 model, switched off in its code for now, and
  its effects), on frames where it draws something. Each renderer sets everything it
  uses (root signature, PSO, topology, vertex buffer, constants, SRV) and hands back a list with
  all of KH1's state marked dirty, so the order of the two renderers does not matter.
- **The pool has a hard limit with no check.** Each close takes the next command buffer from a
  pool of **4 frame slots x 32 buffers** (`0xFD150`; the counters array sits right after the 4 x
  32 x 0x88 bytes). Neither the take (`0xFD150`) nor the increment (`0xF7289`) checks the count;
  the frame end `0x109B40` advances the slot (`(slot+1) & 3`) and clears its counter
  (`0x109DC8`, value in edi, INFERRED 0). A 33rd close in one frame would run into the next slot's
  buffers. How many KH1 uses per frame was not measured. **Guard:** before drawing, read
  `slot = [R+0x5788]`, `used = [R+0x50C8 + slot·4]`; skip our draw (and log once) when `used >=
  24` (ESTIMATE: margin of 8). Log the highest `used` seen.
- **Viewport and scissor are not set by fx12.** The list after a hand-back is new. When it is a
  bundle it inherits viewport, scissor and targets from the direct list that runs it. When it is
  a **direct** list (fx12's `list_type == 0` branch), only the render target is set by fx12;
  D3D12 resets viewports and scissors with the list (INFERRED from the D3D12 rules, not seen
  failing). So if KH2 Combat hands back a direct list and we draw next, we would draw with no
  viewport. **Change:** on a direct list also call `RSSetViewports` (list vtable slot 21) with
  `R+0x15C..+0x168` (depth 0..1) and `RSSetScissorRects` (slot 22) with the frame buffer's
  `0, 0, fb+0x08, fb+0x0C`. On a bundle set neither (not allowed).
- KH2 Combat's `OFF` switch and ours are separate: a failure in one renderer does not stop the
  other.

### 3.4 Detecting KH2 Combat

`kh1-kh2.dll` in our folder or loaded (`hook::wait_for_kh2_combat`). Informational only for the
draw: with the call-site hook nothing depends on it. Log whether `0x1ECF30` starts with `E9`
(KH2 Combat's world-pass hook is in).

### 3.5 Pause menu and second world pass

Copy kh2fx: draw nothing while `[0x232DFB8] != 0`; also skip when a second Sora instance (the
pause menu's copy) was posed in the last 150 ms (our `pose.rs` sees every 293-bone instance; the
field Sora is the player's).

---

## 4. Pipeline states and blend modes

### 4.1 What fx12 has

D3D12 values: blend 1 ZERO, 2 ONE, 6 INV_SRC_ALPHA; op 1 ADD, 3 REV_SUBTRACT. The pixel shader
outputs **premultiplied** colour `(rgb·a, sat(a))`; the alpha channel is not written (mask 0x7:
KH1's destination alpha stays).

| fx12 `Mode` | src, dst, op | result |
|---|---|---|
| Additive | ONE, ONE, ADD | `dst + src·As` |
| Alpha | ONE, INV_SRC_ALPHA, ADD | `lerp(dst, src, As)` |
| Subtract | ONE, ONE, REV_SUBTRACT | `dst - src·As` |
| Darken | ZERO, INV_SRC_ALPHA, ADD (shader outputs black) | `dst·(1 - As)` |

Fixed state: fill SOLID (3), **cull NONE (1)**, depth clip on, MSAA flag when samples > 1;
depth **LESS_EQUAL (4)**, depth enable when the batch asks and a DSV exists, **write mask 0**
for effects (1 only for its opaque model runs); RTV format 87, DSV format 20, sample count and
quality from the frame buffer; one input layout (POSITION R32G32B32, TEXCOORD R32G32, COLOR
R32G32B32A32); triangle lists. PSOs are cached by key.

### 4.2 Design: a blend table keyed by DDD's blend index

Another agent decodes DDD's blend codes (ANM quads: `0x5C02A0` maps 0x70600 -> 1, 0xA0600 -> 2,
0xA0602 -> 3, else 0; every Flowmotion quad and model uses 0xA0600, README 9.4/9.5). The renderer
takes them as data:

```text
struct BlendDesc {            // one row per DDD blend index
    src, dst, op: u32,        // D3D12_BLEND / D3D12_BLEND_OP for colour
    src_a, dst_a, op_a: u32,  // for alpha (default ZERO, ONE, ADD: keep KH1's alpha)
    mask: u8,                 // default 0x7
    premultiply: bool,        // shader outputs rgb·a (true) or straight rgb (false)
    output: Output,           // Colour | Black (fx12's Darken) | ...
}
static BLENDS: [BlendDesc; N]  // index = DDD's blend index; unknown index -> row 2
```

- `PsoKey = { blend: u8, depth_test: bool, depth_write: bool, has_dsv: bool, samples: u32,
  quality: u32 }`. The PSO builder reads the row; nothing else changes in fx12's builder.
- Until the decode lands, row 2 (0xA0600) = Additive (**ESTIMATE**: the shockwaves are glows on
  black-backed noise; to be replaced by DDD's real state). Rows 0, 1, 3 = Alpha (ESTIMATE).
- A row with `premultiply = false` lets DDD's factors be copied as they are (SRC_ALPHA etc.).
- Per-batch shader flags (the 5th root constant, now bit flags): bit 0 textured, bit 1 output
  black, bit 2 cut-out (fx12's), **new** bits 3-4 address mode U, bits 5-6 V (0 wrap, 1 clamp,
  2 mirror; README 13.3: DDD's `+0x24/+0x28 = 2` is a GUESS "wrap", and 2 is MIRROR in D3D11's
  enum, so both must be possible), **new** bit 7 alpha test (GUESS: PSP-style alpha test;
  reference in a 6th constant). Root constants become 41-42 of the 64-DWORD limit (fine).
- **Depth against KH1's scene without writing**: depth enable + write mask 0 + LESS_EQUAL, as fx12.
  KH1's depth is a normal float z/w (0 near, kh1-render.md 2.2), so this needs nothing else.
  Flat rings lying on the floor (the dive's parts 12-15, y = 0 at `Root`) would fight the floor:
  use fx12's per-batch view-z depth offset (`clipk.w`) for them, ESTIMATE 2 KH1 units toward the
  camera, until DDD's own depth state is decoded (README 13.1).
- **Colour**: vertex colour x command colour, 128 = 1.0 each (README 8.1, GUESS), computed on the
  CPU into the f32 vertex colour; the shader multiplies by the texel. Values above 1.0 are kept
  (PSP-style doubling, GUESS) and only alpha is saturated, as fx12 does.

---

## 5. Textures

### 5.1 The 18 textures and their HD replacements (read by `tools/ddd_fep_kh1render_dds.py`)

All 18 have a DDS in `D:/KH-Extract/ddd/remastered/effect/...`: resident k -> `resident.frr/-k.dds`,
`0x9803AC06` -> `sys/ffa/bin/p_fa_ajc0.fep/-0.dds`. **Every one is uncompressed 32-bit, masks R
`00FF0000` G `0000FF00` B `000000FF` A `FF000000` (= B8G8R8A8 byte order), 128-byte header, 1 mip
(no DX10 header).**

| texture | PSP size | DDS size | x | file bytes | RGBA8 bytes |
|---|---|---|---|---|---|
| F0000001 | 128x64 | 512x256 | 4 | 524416 | 524288 |
| F0000004 | 128x64 | 128x64 | 1 | 32896 | 32768 |
| F0000005 | 128x64 | 256x128 | 2 | 131200 | 131072 |
| F0000006 | 128x64 | 128x64 | 1 | 32896 | 32768 |
| F0000007 | 128x128 | 256x256 | 2 | 262272 | 262144 |
| F0000008 | 128x128 | 128x128 | 1 | 65664 | 65536 |
| F0000009 | 128x128 | 128x128 | 1 | 65664 | 65536 |
| F000000C | 128x64 | 256x128 | 2 | 131200 | 131072 |
| F000000F | 128x128 header, decoded 128x80 | 128x80 | 1 | 41088 | 40960 |
| F0000012 | 128x64 | 256x128 | 2 | 131200 | 131072 |
| F0000015 | 256x256 | 512x512 | 2 | 1048704 | 1048576 |
| F0000016 | 128x64 | 256x128 | 2 | 131200 | 131072 |
| F0000018 | 128x64 | 128x64 | 1 | 32896 | 32768 |
| F000001B | 128x64 | 256x128 | 2 | 131200 | 131072 |
| F000001C | 128x64 | 128x64 | 1 | 32896 | 32768 |
| F000001D | 128x64 | 128x64 | 1 | 32896 | 32768 |
| F000001E | 128x64 | 128x64 | 1 | 32896 | 32768 |
| 9803AC06 | 128x64 | 256x128 | 2 | 131200 | 131072 |

Totals: RGBA8 at the DDS sizes **2,990,080 B (2.85 MB)**; at the PSP sizes 950,272 B. The scale
column matches README 9.3's list of the 9 replaced (2x/4x) and 9 identical (1x) textures.

### 5.2 Upload: keep fx12's raw buffer

- **Format**: swap B and R on load (DDS byte order B, G, R, A -> fx12's R, G, B, A); straight
  alpha (README 9.3: the decoders output straight alpha; the 1x DDS equal them bit for bit). No
  mips: neither the PSP textures (`+0x14` = 0) nor the DDS have any, so none are made (DDD
  draws without them).
- **Why not real texture resources**: they need a CBV/SRV/UAV descriptor heap. KH1 records its
  draws into **bundles** much of the time (KH2 Combat says so in fx12's comments; on Steam the
  list getter `0xFD200` opens one through `0xFE540` when its wrapper asks), and
  a bundle may only use the descriptor heaps its direct list has bound: KH1's. A root SRV to a
  buffer needs no heap and works in a bundle. Hardware filtering would need samplers (also a heap).
  So: one `ByteAddressBuffer`, texel offset/width/height per batch, bilinear and address modes in
  the shader, exactly as fx12.
- **Heap**: an upload heap, as fx12 (2.85 MB read by the GPU over the bus; small). Optional later:
  copy once into a default-heap buffer with `CopyBufferRegion` on a direct list.
- **Memory budget**: textures 2.85 MB; vertex ring: fx12's 24 MB is generous. The largest
  Flowmotion models are ~640 strip vertices (`ddd_fep_mdlshape.py` on p_fa_ajc0: 9803AD0E 641,
  9803AD0C/0D 392); as triangle lists about 3x. ESTIMATE peak: the dive's ~40 ring instances x
  ~2000 vertices = 80k vertices = 2.9 MB a frame. Keep 4 x 6 MB (KH1 keeps at most 3 frames in
  flight: upload rings `% 3`, kh1-render.md 3.1). Total GPU-visible memory ~27 MB.

### 5.3 Reading the DDS at run time

Our `egs.rs` was trimmed to original-format files ("no remastered assets"). KH2 Combat's
`crates/kh2-formats/src/egs.rs` has `read_remastered_all(keys, name) -> Vec<(asset name, bytes)>`
(the 0x30-byte HD asset headers after the file header): take it back (NOTICE entry). Read
`effect/resident/bin/resident.frr` and `effect/sys/ffa/bin/p_fa_ajc0.fep` with it from the 2.8
install (`ddd::reader`). Asset names are `-<n>.dds` (the extraction's file names; INFERRED the
same in the archive). **Fallback** without an HD asset: decode the PSP texture with the exact
decoder (port of `tools/ddd_fep_textures.py`: 8x8 Morton tiles, ETC1 base colours not
bit-replicated, README 9.3).

---

## 6. Attachment: bones, units, axes

### 6.1 Which bones the effects use (read by `tools/ddd_fep_kh1render_ead.py`)

From `p_ex01_000.ead` (Sora) and `w_so01_001.ead` (Keyblade), every event that starts one of the
19 Flowmotion elements (82 events; every element is started by at least one):

| DDD bone | elements | KH1 bone (our retarget's pairs, `pose.rs` tests) |
|---|---|---|
| `Root` | AJC SHO0, AKD SHO0, ASK MLC0, COM WAL0/BSS0/MLC0/FIR0, AC MSL0/MSS0, BOS BSS0 | none: the feet under Sora with his facing (actor `+0x10`, yaw `+0x3C`) |
| `mune` (chest) | AJC CAL0, AKD CAL0, AC RNG0 | 76 |
| `hara` (belly) | COM LOC1, BOS LOC1 | 68 |
| `L_ashi2` (left toes) | COM LOC0/LOC1/LOC2 | 49 |
| `center` (hips' fixed bind point, 0.927 up) | AKD DRL0 | none: Root + 92.7 up |
| `Z_00` (Keyblade, 0.779 along `R_buki` +x, ddd-notes) | W_010_SO LOC0 (45 events) | 280 is the grip (`R_buki`); the tip: 77.9 along the grip's blade axis, or KH1's `0x29EB20(actor, 0x30, t = 1, out)` |

**The EAD's own offset (INFERRED from the mirrored pair 030/031)**: after the three name offsets,
an event holds `+0x0C` s16 x3 offset (030 `(-70, 38, 38)`, 031 `(70, 38, 38)`), `+0x18` s16 x3
rotation (030 `(0, 15707, 0)`, 031 `(0, -15707, 0)`: radians x 10000, i.e. +-90 degrees), `+0x1E`
s16 x3 scale (100 = 100 %), `+0x24` RGBA (0x80 each), `+0x28` u16 time in half frames (as
`sound.rs`). Offset unit: **GUESS** 0.01 DDD unit (= 1 KH1 unit). `+0x12`, `+0x14`, `+0x2A..+0x33`
not decoded. This belongs to whoever decodes the EAD; the renderer only needs a `Mat4` from it.

### 6.2 Where the matrices come from each frame

- **While Flowmotion draws a DDD motion** (almost always when these effects start): use **DDD's
  own pose**, the one `pose.rs` draws: `ddd.pose(motion, frame, false)` (Root stays at its bind,
  Flowmotion moves him) gives every DDD bone in DDD model space (y up, DDD units, x his left, z
  ahead). `pose.rs` already keeps `MODEL = (to_world, his position then, hip_ratio)`, and
  `ddd_point()` turns a point on a DDD bone into KH1's world. Add the matrix form:
  `W_bone = to_world · T(hip_ratio · t_P) · R_P` (rotation and placement of DDD bone P, scale 1).
  This covers every bone of 6.1, `Root` and `center` included, and `Z_00` as
  `W(R_buki) · T(77.9 along +x)` (rotation of `R_buki`).
- **When KH1's own animation shows** (an effect outlives its motion): KH1's drawn bone world
  matrices at `instance + 0x1B4` (handle, 0x40 bytes each, 293 bones; `pose.rs` reads them after
  the skeleton pass). Map by the table above. Their axes differ from DDD's bones by the bind
  (`retarget.rs` matches turns relative to each bind), so fold in `bind_KH1⁻¹ · bind_DDD` per
  bone (once). `Root`/`center` from the actor: position `+0x10`, yaw `+0x3C`. ESTIMATE quality:
  only for effects still running after the motion.
- **Snapshot timing**: `pose.rs` runs in the skeleton pass, inside task 140000's model pre-pass
  (`0x1ECB50`, called right after the camera copy at `0x1E0F8A`; INFERRED: Sora's skeleton pass
  is reached from there). The world pass is 160000, so a snapshot taken in `pose::after()` is this
  frame's when we draw.

### 6.3 Units and axes

- **KH1 = DDD x 100** (ddd-notes.md: hips 0.927 / 92.7, thigh, shin, upper arm all x100;
  `retarget` measures `hip_ratio` from the two binds).
- **DDD world/model**: y up, x Sora's left, z ahead (`ddd.rs`): right-handed.
- **KH1 world**: right-handed, **+y down** (kh1-render.md 1.3); KH1 model space is y up, and
  `pose.rs`'s `to_world` maps it to the world by negating the y and z columns of the pass's matrix:
  a 180-degree turn about x, not a mirror. DDD model space = KH1 model space x100, axis for axis
  (`pose.rs` `MODEL` note, checked by the retarget tests). So the whole chain is proper rotations:
  **no handedness flip, triangle winding is kept** (and culling is off anyway).
- **FEP's own y/z negation** (FecLmatrix negates y and z of position and rotation, README 8.2;
  INFERRED: FEP data is authored y down) is part of the interpreter: the `fep` crate copies it, so
  its output is in DDD's y-up element space. The KH1 side must **not** negate again.
- **The element's matrix** handed to the interpreter as its attachment ("effect matrix", DDD's
  instance `+0x60` from `0x5AE260`): `A = W_bone · O_ead · S(100)`, `O_ead` = the EAD offset,
  rotation, scale (6.1). With S(100) inside A, the interpreter keeps working in DDD units and its
  output vertices land in KH1 units.
- **Follow or stay**: DDD recomputes the effect matrix from the attachment every pass-1 update
  (`0x5AE260`, README 7.1), and each part's matrix type decides whether it keeps the matrix it
  was born with (README 8.2, `0x5B10E0`, partly decoded). So supply `A` every tick and let the
  interpreter decide, as DDD does (INFERRED). README 13.4 lists this as open.
- **Camera**: KH1's View (`0x2E9D480`) and P (`[0x233FEA8] + 0x80`, else `0x2340A00`), as fx12's
  `camera()`. Camera-facing parts (Lmatrix byte 1, INFERRED) need the camera in the element's
  space: give the interpreter the KH1 View; it takes `A⁻¹` itself.

---

## 7. Timing

- **KH1**: game logic at 60 fps; the frame step `0x233FBDC` is 1.0 at 60 fps and 2.0 at 30 (KH2
  Combat says; a float read in 426 places). It can take other values for a while (`0x18D307`
  stores 5.0; `0x17F590` saves and restores it).
- **DDD's FEP clock**: `dt = 0x6294E0() x 0.5` per frame (README 7.1); if `0x6294E0` returns 1.0
  per 60 fps frame (README's **GUESS**), one FEP frame = 1/30 s and DDD at 60 fps steps 0.5.
  Flowmotion's motions already run at `k x 0.5` motion frames per game frame (`flow.rs`), and the
  EAD times are in those motion frames.
- **So: `dt_fep = frame_step x 0.5` per KH1 game frame** (0.5 at 60 fps). Clamp to 0.25..2.0
  (ESTIMATE) against the odd step values above; the interpreter does DDD's own dt > 10 rule.
- **Where**: count, step where the bones are fresh.
  1. In the physics-step hook, for the player only (`flow::after_player_step`, which already
     calls `sound::frame`): trigger EAD events by the same `reached(t, f, prev)` rule as
     `sound.rs`, queue element starts, and add `dt_fep` to a pending total. No physics frame (pause,
     full-screen menus) = no time: effects freeze, as kh2fx's do.
  2. In the world-pass hook: take the pending dt, refresh every running element's `A` from this
     frame's snapshot, run the interpreter in steps of at most 0.5 (DDD's own granularity at 60
     fps), build triangles with this frame's camera, draw.
  This gives DDD's own sub-frame time (0, 0.5, 1.0, ...: keys at `int(time)`, mv/ac forms at the
  exact f32 time), bones and camera of the frame being drawn, and no double step when the world
  pass runs twice (the second pass finds nothing pending; it is skipped anyway, 3.5).
- **High frame rates**: KH1 can replay a recorded frame with an interpolated camera (`0x1025A0`,
  from the frame end and the pacing thread). Commands we record straight into the list are not in
  that replay (kh1-render.md 3.2; KH2 Combat draws KH1's model instead of its own while replays
  happen). INFERRED: with interpolation on, our effects show on real frames only (flicker at the
  display rate). Plan: hook `0x1025A0` only to count replays (KH2 Combat hooks it too: same rules,
  3.1; or patch its two call sites) and log; which setting turns it on is not known.

---

## 8. Module plan

### 8.1 Files

| file | new / changed | what |
|---|---|---|
| `src/fx/mod.rs` | new | the public API and the state (one `Mutex<Option<State>>`, `try_lock` from hooks): `load()`, `install(game)`, `frame(now, fresh)`, `draw(base)`, `stop_all()` |
| `src/fx/kh1.rs` | new | every KH1 address of section 2 as Steam RVAs with expected bytes (checked once at start, like `game.rs` SIGNATURES); `camera()`, `clip_constants(r)`, `target(r)`, `lists_used(r)`, `paused(base)`, `frame_step(base)`, `viewport(r)` |
| `src/fx/gpu.rs` | new, **adapted from fx12.rs** | D3D12: COM layouts, `init(device)`, `pso(PsoKey)`, `upload()`, `begin()` (+ viewport/scissor on direct lists), `hand_back()`, `draw(base, &Frame, &Camera) -> Result<Drawn, Skip>`, `set_textures()` |
| `src/fx/blend.rs` | new | `BlendDesc`, `BLENDS` table keyed by DDD's blend index (4.2) |
| `src/fx/texture.rs` | new | DDS reader (the header of 5.1; B8G8R8A8 32-bit only, anything else refused and logged), PSP TEX decoder fallback, `Texture { id, w, h, rgba }` |
| `src/fx/anchor.rs` | new | `Bone` (DDD name -> index; KH1 fallback), `Anchor { bone, offset: Mat4 }`, `Snapshot`, `matrix(anchor, &snapshot) -> Option<Mat4>` = `A` of 6.3 |
| `src/fx/trigger.rs` | new | EAD events per motion (file, element, bone, offset, time) for Sora and the Keyblade; `due(motion, f, prev)` |
| `src/fx/hook.rs` or `src/hook.rs` | new code | the call-site patch at `0x1E0FAD` (3.2) and the `RawDetour` fallback |
| `src/egs.rs` | changed | `read_remastered_all` back from KH2 Combat's `kh2-formats/src/egs.rs` |
| `src/pose.rs` | changed | in `after()`: store the anchor snapshot (DDD pose of the drawn motion, `to_world`, ratio, and KH1 bones 0, 49, 68, 76, 280 as drawn); note the pause menu's Sora copy (a 293-bone instance that is not the player's) |
| `src/sound.rs` | changed | `reached()` shared with `fx::trigger` (or moved to a small `src/events.rs`) |
| `src/flow.rs` | changed | `crate::fx::frame(crate::pose::requested(), fresh)` beside `sound::frame` |
| `src/lib.rs` | changed | after `load_ddd`: `fx::load(...)` on a thread, then `fx::install(game)` |
| `Cargo.toml` | changed | `fep = { path = "fep" }` (the interpreter crate, another design); windows-sys features `Win32_System_Memory` (VirtualAlloc/VirtualProtect/VirtualQuery), `Win32_System_Diagnostics_Debug` (FlushInstructionCache), `Win32_System_Threading` (GetCurrentProcess) |
| `NOTICE` | changed | entries for `gpu.rs` (from fx12.rs) and the remastered reader (from kh2-formats egs.rs) |

### 8.2 The boundary with the `fep` crate

The renderer needs from the interpreter only what kh2-fx gives fx12, with DDD's state:

```text
// in the fep crate (platform-free); names are a proposal
pub struct Vertex { pub pos: [f32; 3], pub uv: [f32; 2], pub color: [f32; 4] }  // 36 bytes, KH1 world, 1.0 = full
pub struct Batch {
    pub texture: Option<u32>,   // index into Library::textures()
    pub blend: u8,              // DDD's blend index (4.2)
    pub depth_test: bool, pub depth_write: bool,
    pub depth_offset: f32,      // view-z shift, KH1 units
    pub address: [u8; 2],       // U, V: 0 wrap, 1 clamp, 2 mirror
    pub alpha_ref: Option<u8>,  // alpha test, if DDD has one (GUESS)
    pub first: u32, pub count: u32,   // triangle list in Frame::vertices
}
pub struct Frame { pub vertices: Vec<Vertex>, pub batches: Vec<Batch> }

impl Player {
    fn start(&mut self, file: &str, element: &str, anchor: Mat4) -> Option<Handle>;
    fn set_anchor(&mut self, h: Handle, anchor: Mat4);        // A of 6.3, every tick
    fn stop(&mut self, h: Handle);                            // DDD's stop request (body +0x0C)
    fn tick(&mut self, dt: f32);                              // one DDD frame step (<= 0.5 here)
    fn build(&self, view: Mat4, out: &mut Frame);             // draw lists -> KH1-world triangles
    fn running(&self) -> usize;
}
// Library::textures() -> [(ddd id, w, h, Option<rgba>)]: the renderer fills rgba from DDS or PSP data
```

Batches come in DDD's draw order (element, then the part lists, the two draw passes of README
7.1); the renderer draws them in that order.

### 8.3 Data flow

```text
start-up (thread, lib.rs)            each player physics step (game thread)        world pass (game thread, task 160000)
-------------------------            --------------------------------------        -------------------------------------
2.8 install -> FEP, ESE, frr, EAD    flow::after_player_step                       KH1's pass (and KH2 Combat's draws)
  -> fep::Library                      -> fx::frame(motion, frame, fresh)           -> fx::draw(base)
remastered DDS / PSP TEX                  EAD events reached -> starts queued          paused / menu copy? -> return
  -> RGBA8 by texture index               pending_dt += frame_step x 0.5            snapshot (pose.rs, this frame)
  -> gpu::set_textures                                                                -> A for every running element
                                     skeleton pass (pose.rs, task 140000)            starts -> Player::start(A)
                                       Sora's matrices written                       while pending_dt > 0: tick(min(0.5, ..))
                                       -> anchor snapshot stored                     camera = kh1::camera()
                                                                                    Player::build(view) -> Frame
                                                                                    gpu::draw: budget guard, PSOs, ring,
                                                                                      list, state, draws, hand_back
```

### 8.4 What changes in fx12.rs (for the file header and NOTICE)

Kept: the `com!` macro, every COM struct layout and its size asserts, the root signature, the HLSL
(vertex shader and the bilinear texel fetch), `buffer()`, the ring `upload()`, `camera()`,
`constants()`, `target()`, `begin()`, `hand_back()`, the fail-once switch, the crash note.
Changed:

1. Addresses: Steam RVAs directly (no `Rva`/`steam_map.in`), each checked by bytes at start.
2. `Mode` (4 PS2 modes) -> `blend: u8` and the `BLENDS` table (4.2); `PsoKey` with it.
3. Shader flags: address mode per axis, optional alpha test (6th constant).
4. `begin()`: on a direct list also set viewport and scissor (3.3).
5. Budget guard on KH1's command-buffer pool before drawing (3.3).
6. Input types: our `Frame`/`Vertex`/`Batch` (the fep crate's) instead of `kh2_fx`'s; texture
   input from DDS (B/R swapped) or PSP decodes.
7. Windows calls through `windows-sys` instead of KH2 Combat's `sys` module; `readable()` via
   `VirtualQuery`.
8. `draw_mesh` and the model arena dropped (not needed).
9. Log text and names ("FEP effects" instead of "KH2 effect").

---

## 9. Risks, most serious first

1. **KH1's command-buffer pool overflows** (32 per frame slot, no bound check, 3.3): three
   renderers' hand-backs on top of KH1's own. Mitigation: the guard and a log of `used`.
2. **Hook order** (3.1): if we hook the prologue first, KH2 Combat silently loses its world-pass
   hook. Mitigation: the call-site patch (3.2); the fallback waits for KH2 Combat's `DllMain`.
3. **Direct list without viewport** after another renderer's hand-back (3.3). Mitigation: set
   viewport and scissor on direct lists.
4. **Interpolated frames** (7): effects missing on replayed frames. Unknown setting; counted and
   logged first.
5. **Draw semantics not decoded** (README 13.1-13.3): blend state, colour scale, depth/sort, UV
   address mode. The renderer takes them as data (4.2); defaults are ESTIMATEs and will look
   wrong until the decode lands.
6. **Floor fighting** of the ground rings: the depth offset is an ESTIMATE.
7. **Time base** (7): `0x6294E0`'s unit is a GUESS; if wrong, every effect runs 2x fast or slow.
8. **Attachment details**: the EAD offset units (GUESS), follow vs stay (INFERRED from
   `0x5AE260`), the KH1-bone fallback quality (ESTIMATE).
9. **Frame-buffer fields not re-read on Steam** (2.2): DSV id/heap/increment, MSAA count/quality.
   fx12's first-draw log line shows them; a wrong one gives a refused PSO or no depth test, logged.
10. **Remastered assets in the archive**: if the 2.8 install's PKG lacks them (or names differ),
    the PSP decodes are drawn (DDD's look on 3DS, not PC). Logged.
11. **KH1 makes a new device** (window mode change): fx12 turns itself off ("KH1 made a new
    device"); keep, effects stop for the session.
12. **d3dcompiler**: present on Windows 10/11 (fx12 tries _47, _46, _43). Alternative: embed
    precompiled DXBC (needs fxc at build time).

---

## 10. Open points (most important first)

1. DDD's blend, depth, sort and sampler state per blend index (another agent): fills `BLENDS`
   and the batch flags.
2. Runtime counts: KH1's command buffers used per frame (`R+0x50C8[slot]`), bundle vs direct at
   our draw, the frame buffer fields (one log line each).
3. Whether the interpolated replay is on at the user's settings, and what to do if so.
4. EAD event fields past the three names (offset units, `+0x12`, `+0x2A..+0x33`).
5. The unit of DDD's `0x6294E0` (the FEP time base).
6. Follow vs stay per part matrix type (`0x5B10E0`, README 13.4).
7. KH1-bone fallback axes (bind differences per bone) if effects outlive their motion.
8. Asset names of the remastered DDS inside the 2.8 PKG (`-<n>.dds` assumed from the extraction).
9. Whether the second world pass (pause menu) is fully covered by the pause state check.

---

## 11. Tools written for this doc

| tool | what |
|---|---|
| `tools/ddd_fep_kh1render_verify.py verify` | the 27 checks of section 2 on the Steam KH1 exe (PE stamp, prologues, call sites, registrations, RIP-relative and displacement references, decoded from `.pdata` function starts); subcommands `dis`, `refs`, `disp`, `func`, `callers`. Last output kept at `a local work folder` |
| `tools/ddd_fep_kh1render_dds.py` | DDS headers of the 18 Flowmotion textures (section 5.1), with the PSP sizes from resident.frr / p_fa_ajc0.fep |
| `tools/ddd_fep_kh1render_ead.py [--all]` | every EAD event that starts a Flowmotion element: motion, time, file, element, bone, raw fields (section 6.1); output kept at `a local work folder` |
