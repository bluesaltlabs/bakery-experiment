# Bakery Puzzle-Sim

A 2D top-down bakery production puzzle game inspired by Chip's Challenge. Move around a grid-based bakery floor, interact with production stations, process gluten-free pizza crust through a simplified production line, and complete a case goal before time runs out.


## Controls

| Key | Action |
|-----|--------|
| W / Arrow Up | Move up |
| S / Arrow Down | Move down |
| A / Arrow Left | Move left |
| D / Arrow Right | Move right |
| E / Space | Interact (pick up, drop, insert/collect from station) |
| R | Restart game |
| G | Toggle grid lines |

## Production Chain

1. **Source** (green) — Automatically spawns DoughBatch every 4s
2. **Former** (orange) — Processes DoughBatch → RawCrustTray (3s)
3. **Oven** (red) — Processes RawCrustTray → BakedCrustTray (5s)
4. **Packer** (blue) — Collects 3 BakedCrustTray → produces 1 Case (2s)
5. **Palletizer** (purple) — Accepts Case → increments completed case count

### Goal

Deliver 10 cases to the Palletizer within 300 seconds.

### Loss

Time runs out before reaching 10 cases.

## Win/Loss Conditions

- **Win**: Cases completed ≥ 10 before timer expires
- **Loss**: Timer reaches 0 without meeting the target


## Build

To build for WASM, run:

```bash
./wasm-build.sh
python -m http.server 8081 -d web
```


Otherwise just run:

```bash
cargo run
```


---
