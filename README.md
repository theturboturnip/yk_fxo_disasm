# yk_fxo_disasm

This crate (will) disassemble `.fxo` files used in the PC ports of games from the Ryu Ga Gotoku (Like A Dragon, or Yakuza) game series.

It (will) make use of the [`amd_dx_gsa`](https://github.com/theturboturnip/amd_dx_gsa) and [`turnip_gfx_disasm`](https://github.com/theturboturnip/turnip_gfx_disasm) crates to handle data extracted from the `.fxo` files.

The main process ~~is~~ (will be)
1. Extract the vertex/fragment/geometry? DirectX ByteCode bytes from a `.fxo` file
2. Use `amd_dx_gsa` to compile that DXBC to a AMD RDNA2 binary
3. Use `turnip_gfx_disasm` to perform dependency tracking on the AMD RDNA2 binary
4. Use these dependencies, plus knowledge of the relevant game's rendering pipeline, to understand which inputs to the vertex shader affect which outputs in the fragment shader.

## Resources

This requires:
1. A legitimate PC install of a RGG game based on the Dragon Engine (e.g. Yakuza 6, Yakuza Kiwami 2, Yakuza: Like A Dragon, Judgment, Lost Judgment)
   - Older engines use separate `.vso` and `.pso` files, so shouldn't be difficult to support, but they will not be supported initially.
   - Future games may be based on the Unreal Engine (e.g. Like A Dragon: Ishin), which this will (probably) not be compatible with.
2. An application for extracting the `.fxo`s from `.par` files (i.e. [ParTool](https://github.com/Kaplas80/ParManager))
3. `atidxx64.dll`, which is not a redistributable file. See below for installation instructions.

## Usage Instructions

TODO - this program doesn't exist yet lol

### `atidxx64.dll`
If you have an AMD GPU this may already be installed somewhere on your computer.

If you have RenderDoc installed, it may be installed in `C:\Program Files\RenderDoc\plugins\amd\isa\atidxx64.dll`.

Otherwise, follow [the RenderDoc wiki instructions to acquire it.](https://github.com/baldurk/renderdoc/wiki/GCN-ISA#d3d11-and-d3d12-disassembly-with-amd-driver)
