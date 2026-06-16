# Bakery Puzzle-Sim

A 2D top-down bakery production puzzle game inspired by Chip's Challenge. Move around a grid-based bakery floor, interact with production stations, process gluten-free pizza crust through a simplified production line, and complete a case goal before time runs out.

## MVP Scope

- Top-down 2D grid
- One player
- One small level
- One production chain (Dough → Raw Tray → Baked Tray → Case)
- One countdown timer
- One case target (3 cases to win)
- Placeholder graphics (colored squares)

## Controls

| Key | Action |
|-----|--------|
| W / Arrow Up | Move up |
| S / Arrow Down | Move down |
| A / Arrow Left | Move left |
| D / Arrow Right | Move right |
| E / Space | Interact (pick up, drop, insert into station) |
| R | Restart game |

## Production Chain

1. **Source** (green) — Automatically spawns Dough
2. **Former** (orange) — Processes Dough → Raw Crust Tray (3s)
3. **Oven** (red) — Processes Raw Tray → Baked Crust Tray (5s)
4. **Packer** (blue) — Collects 3 Baked Trays → produces Case (4s)
5. **Palletizer** (purple) — Accepts Case → increments score

### Goal

Deliver 3 cases to the Palletizer within 120 seconds.

### Loss

Time runs out before reaching the target.

## Win/Loss Conditions

- **Win**: Deliver `target_cases` (3) to the Palletizer before time expires
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

## Future Ideas

- Multiple levels with increasing complexity
- Product recipes and branching production lines
- Forklift vehicle for faster transport
- NPC workers to automate tasks
- Score tally and time bonuses
- Sound effects and music
