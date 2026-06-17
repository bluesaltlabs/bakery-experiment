# Bakery Puzzle-Sim

A 2D top-down bakery production puzzle game inspired by Chip's Challenge. Move around a grid-based bakery floor, interact with production stations, process gluten-free pizza crust through a simplified production line, and complete a case goal before time runs out.

## MVP Scope

- Top-down 2D grid (12×10 tiles)
- One player (grid-based movement with wall collision)
- One hardcoded level with routing tension
- One production chain (Source → Former → Oven → Packer → Palletizer)
- Shift countdown timer (300 seconds)
- Case target (20 cases to win)
- Placeholder graphics (colored rectangles)
- HUD showing timer, score, and carried item
- Win/loss states with restart (R key)

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

Deliver 20 cases to the Palletizer within 300 seconds.

### Loss

Time runs out before reaching 20 cases.

## Win/Loss Conditions

- **Win**: Cases completed ≥ 20 before timer expires
- **Loss**: Timer reaches 0 without meeting the target

## Non-Goals (excluded from MVP)

- Multiple levels
- Product-code switching
- Forklifts
- NPC workers
- Sound / music
- Save / load
- Advanced UI
- Networking
- Real-world pallet physics
- Conveyor belts
- Station jams or maintenance events
- Animation

## Level Layout

The bakery floor is a 12×10 tile grid with walls (grey), floor (light grey), and five production stations. A central wall divider creates routing tension — the player must navigate around it to move items between the left side (Source, Former) and right side (Packer, Palletizer).

| Tile | Color | Position |
|------|-------|----------|
| Floor | Light grey (0.5, 0.5, 0.55) | — |
| Wall | Dark grey (0.3, 0.3, 0.35) | Divides left/right corridors |
| Source | Green (0.2, 0.8, 0.2) | Top area |
| Former | Orange (0.8, 0.5, 0.2) | Lower left |
| Oven | Red-orange (0.9, 0.3, 0.1) | Lower center |
| Packer | Blue (0.3, 0.3, 0.8) | Lower right |
| Palletizer | Purple (0.8, 0.2, 0.8) | Bottom right |
| Player | Blue (0.3, 0.6, 1.0) | Starts near center |

## Architecture (for developers)

- **ECS-based** with Bevy 0.14
- `components.rs` — All component types (GridPos, Player, Facing, Carrying, Item, Station)
- `resources.rs` — ShiftState (timer, score, game_over, victory) and MovementCooldown
- `level.rs` — Hardcoded level data, grid↔world conversion, level setup
- `player.rs` — Player entity spawning
- `movement.rs` — Grid-based movement with wall/station collision
- `interaction.rs` — Pickup, drop, station insertion/collection
- `stations.rs` — Processing timers, source auto-spawn, ground-item sync
- `ui.rs` — HUD text, game-over overlay, restart handler, shift timer

## Future Ideas

- Multiple levels with increasing complexity
- Product recipes and branching production lines
- Forklift vehicle for faster transport
- NPC workers to automate tasks
- Score tally and time bonuses
- Sound effects and music
- Station jams and blocked outputs
- Shift report / score screen
