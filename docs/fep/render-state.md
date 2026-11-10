# DDD effect drawing: the GPU side (render state, shaders, sort)

How DDD PC (KINGDOM HEARTS Dream Drop Distance.exe, Steam, KH 2.8) puts the FEP model effects on
screen: API, blend, depth, cull, alpha test, sampler, the shader math, and the draw order. Scope:
what the shock models (FecMdl_plus / FecMdlts_plus, blend index 2) need; the mesh side is in
`draw-models.md`. Every address is an RVA (image base 0x140000000). **Labels:** unmarked = read
from the exe or the files; **INFERRED** = follows from read facts; **GUESS** = not read.
Tools: `ddd_fep_renderstate_programs.py` (shader program table), `ddd_fep_renderstate_shaders.py`
(DXBC extraction/disassembly; output in `%TEMP%/khfm/fepwork/renderstate/shaders/`),
`ddd_fep_renderstate_data.py` (per-element state survey).

## 0. Summary for a renderer (blend index 2, the shock models)

    colour = tex.rgb * vcol.rgb * K.rgb       (vcol: 0x80 = 1.0; K: 0x80 = 1.0)
    alpha  = tex.a   * vcol.a   * K.a
    discard if alpha <= 0.5/255                (alpha test GREATER, ref 0)
    dest.rgb = dest.rgb + colour * alpha       (D3D12 ADD, SRC_ALPHA, ONE)
    depth test LESS_EQUAL on, depth write off, no culling,
    sampler: wrap U/V, linear (4x anisotropic) filtering

## 1. API and backend

- Imports: `d3d12.dll` (ordinal 101 = `D3D12CreateDevice`, `D3D12SerializeRootSignature`),
  `dxgi.dll` (`CreateDXGIFactory1`), `D3DCOMPILER_47.dll` (`D3DStripShader` only),
  `d3d11.dll` (`D3D11CreateDevice` only). **DDD PC draws with Direct3D 12.**
- On top of D3D12 sits a GL-like context (returned by `0x6530e0`) fed by 3DS-style state objects.
  The effect code fills a **fragment-operation block** at `0xaaa360` (returned by `0x5e19a0`; field
  `+0x80` = dirty bits) and flushes it with `0x5e2ea0`, whose handlers call the context setters:

  | dirty bit | block field | handler | context field |
  |---|---|---|---|
  | 1 | `+0x00` blend | `0x66f610` | `+0x180` colour blend, `+0x1a8` alpha blend |
  | 4 | `+0x18` alpha test | `0x66f5b0` | PSFLAGS / alphaRef (`0x65cb80`, `0x65caf0`) |
  | 0x20/0x10 | `+0x40` depth | `0x66f780` | `+0x260` enable, `+0x270` write, `+0x264` func |
  | 0x40 | `+0x58` cull | `0x66f720` | `+0x84` D3D12 cull mode |
  | 8, 2, 0x80 | `+0x20` stencil, `+0x0c`, `+0x64` | `0x66fa10`, `0x66f850`, `0x66fd70` | not used here |

- The PSO for a draw is keyed in `0x649160` (`0x649fde..0x64a3df`) from these context fields and
  the program id (`ctx+0x60`, set by `0x65dfc0`).
- The block's defaults (`0x5e2530`, applied to the two preset blocks at `0xaaa000`/`0xaaa1b0` by the
  frame setup `0x5dee8f`/`0x5deea0`, copied to `0xaaa360` by `0x5e0440`): blend off (index 1 values
  stored), alpha test on / GREATER / ref 0, depth test on / write on / LESS_EQUAL, cull on (FRONT),
  `+0x64` = 0.01. The effect draws set only blend, depth write and cull; the other fields keep
  whatever the block holds (INFERRED: these defaults, since no effect code writes them).

**Enums are D3D12's own**: blend factors/ops are packed 4/3 bits into the PSO key and compared with
14/15 (= `D3D12_BLEND_BLEND_FACTOR` / `INV_BLEND_FACTOR`) to decide whether the blend constant
matters (`0x64a011..0x64a01f`); depth and alpha functions are stored as `func - 1` in 3 bits
(`0x64a35e`, `0x65cb04`) and the shader's alpha-test decode (section 3) matches the
`D3D12_COMPARISON_FUNC` numbering; the sampler loop (section 6) writes `D3D12_SAMPLER_DESC` values.

## 2. Blend (selector -> index -> D3D12)

Index from the `_plus` selector or the MDL code: `0x5c02d0` / `0x5c02a0` (draw-models.md 6).
`0x5e2300(block, index)` writes the block (bytes: `+0` enable, `+1` colour op, `+2` alpha op,
`+3` colour src, `+4` alpha src, `+5` colour dst, `+6` alpha dst); `0x66f610` passes
(`+1`, `+3`, `+5`, `+8`) to `0x65cc30` (context `+0x184` = {op, src, dst, constant}) and
(`+2`, `+4`, `+6`, `+0xb`) to `0x65cbd0` (`+0x1ac`); `+0x180`/`+0x1a8` = enable.

| index | source | colour (op, src, dst) | alpha (op, src, dst) | result |
|---|---|---|---|---|
| 0 | `0x5e2304` | blend off | blend off | opaque write |
| 1 | `0x5e231e` (selector 0, 3, 5-7; code 0x70600) | ADD, SRC_ALPHA, INV_SRC_ALPHA | ADD, ONE, ZERO | normal alpha blend |
| **2** | `0x5e2334` (selector 1; code 0xA0600) | **ADD, SRC_ALPHA, ONE** | ADD, ONE, ZERO | **additive, weighted by alpha** |
| 3 | `0x5e2329` (selector 2; code 0xA0602) | REV_SUBTRACT, SRC_ALPHA, ONE | REV_SUBTRACT, ONE, ZERO | subtractive: dest - src x alpha |

`D3D12_RENDER_TARGET_BLEND_DESC` for index 2: BlendEnable TRUE, SrcBlend 5 (SRC_ALPHA), DestBlend 2
(ONE), BlendOp 1 (ADD), SrcBlendAlpha 2 (ONE), DestBlendAlpha 1 (ZERO), BlendOpAlpha 1 (ADD);
LogicOp off. Colour write mask: from `ctx+0x274` (`0x66f780` -> `0x65d5b0`), **INFERRED all
channels** for effects (the effect code never touches it). The destination alpha is overwritten
with src alpha (ONE, ZERO): harmless, the back buffer alpha is not displayed (INFERRED).

Order independence: with index 2 and no depth write every shock draw only adds a non-negative
amount, so the composite does not depend on draw order (saturation at 1.0 is also order
independent for non-negative terms).

## 3. Shader program and combiner formula

The "texture environment" is not a PICA TEV: it is a small preset `{u32 K, u32, u32 A, B, C, D,
u8, u8}` (`0x6723f0`) selected by `0x5e29a0` and copied to `0x14db9220..` by `0x66f6a0`; the draw
setup `0x670db0` turns it into shader constants and a program id.

- Presets (in the 0x1b0-byte state block, `0x5e2530`): textured `+0xdc` (`0x5e2b50`: A 0x101,
  B 0x202, C 0x202, D 0, K 0x80808080), untextured `+0xf8` (`0x5e2b90`: A 0, B 0x201, C 0x202).
  The model draw writes K = element colour into the preset it uses (`0x5e2a09`).
- Program (`0x670db0`, vertex mode 5, `0x671031..0x6710bd`): **0x27** = VS blob 23 (`0x809cd0`) +
  PS 14 = blob 42 (`0x814380`); D != 0 would select 0x29 (PS blob 43). Program table: `0x655160`,
  `ddd_fep_renderstate_programs.py`.
- Constants (`0x65ce90(id)` -> cbShaderParam slot id, `ctx+0x5ba8 + 16 id`; 0x1001 -> slot 1):
  - `Ambient` (slot 1) = K / 128 per channel, because C = 0x202 (`0x6711f8..0x671289`: low byte 2
    -> RGB / 128, high byte 2 -> A / 128; otherwise / 255).
  - `TexScale` (slot 0) = UV scale/offset (draw-models.md 7).
  - `ParamVector[0]` (slot 3) = default vertex colour (128/255 each) for formats without colour;
    `ParamVector[1]` (slot 4) = (1 if the format has colour else 0, 0, 0, 0) (`0x6712d7..0x671369`).
  - `colorScale` / `alphaScale` (cbConstantVS +64/+68, `ctx+0x5a20/0x5a24`, `0x6497c5..0x6498c5`)
    = **255/128 = 1.9921875** when B's low / high byte is 2 (`0x6711a4..0x6711b6` -> `ctx+0x290`,
    `+0x294`). B = 0x202 -> both x1.992.
- VS blob 23 (colour part):

      c = lerp(ParamVector[0], vertex.rgba, ParamVector[1].x) * Ambient
      out.rgb = c.rgb * colorScale ;  out.a = c.a * alphaScale   (* height cut, below)
      uv = vertex.uv * TexScale.xy + TexScale.zw  (then * TPage = 1)

  So **vertex colour 0x80 = 1.0** (128/255 x 255/128) and **K 0x80 = 1.0**; a vertex format
  without colour behaves as vertex colour 0x80. The height cut (`ParamVector[4]`: alpha 0 below
  y, fade above x) is reset to (10000, -100000) by `0x66f4d3..0x66f4eb`: off (INFERRED for effects).
- PS blob 42: `t = tex(uv); t.a = PSFLAGS bit 24 ? extTexAlpha : t.a; c = t * vs_colour;`
  alpha test (below); fog mix `c.rgb = lerp(FogColor, c.rgb, fog)` when the VS fog factor != 1;
  output c. **No x2 in the pixel shader**; no clamp before blending other than the render target's.
- **Fog**: VS fog = FogEnable (cbConstantVS +52, `ctx+0x5a14` <- `ctx+0x280`, set by `0x66fe80`)
  ? saturate((w - FogParam.y)/(FogParam.x - FogParam.y)) : 1. The effect code does not set it;
  **open** whether scene fog is on during the effect pass (GUESS off, as in KH1 additive effects).

Full formula for a shock fragment (vertex format with colour, textured):

    rgb = tex.rgb * (vcol.rgb/255 * 255/128) * (K.rgb/128)
    a   = tex.a   * (vcol.a  /255 * 255/128) * (K.a  /128)

## 4. Depth

- Test: enabled by default (`0x5e2585`), function 4 = **LESS_EQUAL** (`0x5e2593`). The effect
  passes do not change it, except the passes for sort groups 7 and 8 (`0x5572a0`, `0x5572e9`:
  depth test off).
- Write: `+0x44` = !(texref.flags bit 2) per sub (`0x5be629..0x5be634`); flags 7 -> **off**.
- **Depth push** (Mdlts `+0x48`, draw-models.md 8): a per-draw clip-space depth offset in the VS
  (`0x66fe00`: ParamVector[3].x = shift x 3.8147e-06 (1/262144) when `[[0xaa3e40]+0x28]` = -1;
  VS blob 23 `z += ParamVector[3].x * w`). Not used by the shock parts. The VS's other
  ParamVector[3].y branch is the 3DS w-buffer emulation (default -1 = off, `0x66f4b6`).

## 5. Cull

`+0x58` = texref.flags bit 3 (`0x5be684..0x5be68a`); 0 -> `0x66f720` calls `0x65d190(1)` ->
**D3D12_CULL_MODE_NONE**. (When on: mode = 2 + (front xor back flags), FRONT or BACK.) All
Flowmotion texrefs have flags 7 -> no culling: the rings are seen from both sides.

## 6. Sampler

- `0x5c0710(mgr, cmd, slot, 5, 1, 0, 0)` -> `0x5c07f0` -> `0x66faa0`: the address modes come from
  the texture object (`+0x24`/`+0x28`, both set to 2 at upload `0x5fd06f`/`0x5fd076`) through the
  table {0,1 -> clamp; 2,3 -> wrap} (`0x66fb10..0x66fb21`, then `0x649c62..0x649c8e`) into
  `ctx+0x1f0` (from texture `+0x28`) and `ctx+0x204` (from `+0x24`).
- Sampler heap (`0x65b6aa..0x65b786`): 8 descriptors, index = (U x 2 + V) x 2 + f with
  U = `ctx+0x1f0` -> AddressU, V = `ctx+0x204` -> AddressV, values
  AddressU/V from {**1 = WRAP**, 3 = CLAMP}, AddressW WRAP, f 0 -> Filter 0x15
  (MIN_MAG_MIP_LINEAR), f 1 -> 0x55 (ANISOTROPIC, MaxAnisotropy 4); MipLODBias 0, ComparisonFunc
  NEVER, border 0, MinLOD 0.
- f = !(program `+0x14` bit 1) (`0x649ca2..0x649cb4`); program 0x27 has `+0x14` = 1 -> f = 1.
- **Shock textures: WRAP / WRAP, ANISOTROPIC x4** (equivalent to bilinear on a texture without
  mips). The texture's own filter fields (`+0x2c` = 1, `+0x30` = 3) only reach `0x672310`, which
  stores a flag the D3D12 path does not use (INFERRED).
- Textures: RGBA8, straight (not premultiplied) alpha (textures.md); the HD DDS replacements are
  used as-is (README 9.3).

## 7. Draw order and sort groups

- Every drawn part is linked into a bucket list (`0xaa3e50 + (screen x 0x140 + bucket) x 8`) by
  `0x5bc450` (called from Lmatrix `0x582590`). Bucket: Lmatrix byte 4 = 0 -> the element instance's
  `+0x14c` (set by whoever spawned the element, **open**), or depth-sorted (`0x58c550`,
  `0x5bbdc0`) when the body's `+0x0a` = 3 (0 for all Flowmotion bodies); byte 4 != 0 -> tables at
  `0x9e8638`/`0x9e8648`/`0x9e85c8`. All shock parts have bytes 4, 5 = 0.
- The passes `0x557390`, `0x5573f0`, `0x557230`, `0x557470`, `0x5572a0` draw groups 0..8 through
  `0x5bc060` (group -> buckets: 0: 0; 1: 1-8; 2: 9-0x108; 3: special `0x5bc200`; 4: 0x10a-0x111;
  5: 0x112-0x119; 6: 0x11a-0x121; 7: 0x12a-0x131; 8: 0x132-0x137; 10: 0x138-0x13f). Only the
  pass for groups 7/8 changes render state (depth test off).
- **For the shock models the order does not matter** (section 2). It would matter only for the
  depth test, if the spawner put them into groups 7/8 (open; GUESS no: they are ground effects
  that must be hidden behind walls).

## 8. Open points

- Which bucket the dive/slam elements get (instance `+0x14c`, set by the spawner): decides only
  whether the depth test is on (section 7).
- Whether scene fog is enabled during the effect passes (section 3).
- The colour write mask (`ctx+0x274`), INFERRED RGBA.
- Lmatrix byte 1 = 2 (slam cups, parts 7/8): the rest of `0x5b1dc9..0x5b2137` (draw-models.md 8).
