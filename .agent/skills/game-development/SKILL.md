---
name: game-development
description: Game development orchestrator. Routes to platform-specific and genre-specific game engine reference guides.
when_to_use: "When building games with Unity, Godot, Unreal, Phaser, or any game engine. Routes to platform-specific reference guides."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, RunCommand
version: "2.1.0"
priority: low
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / game-development
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Game Development Orchestrator

> **Role**: Core game architecture and routing hub for platform-specific and genre-specific game reference guides.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core loop rules below; consult deep **L3 Reference Guides** on demand:

| Domain / Target | Reference Guide | Focus / Technologies |
|---|---|---|
| **Web Games** | [`references/web_games.md`](./references/web_games.md) | HTML5, Canvas, WebGL, PixiJS, Three.js, Phaser |
| **Mobile Games** | [`references/mobile_games.md`](./references/mobile_games.md) | Touch controls, battery optimization, iOS/Android |
| **PC Games** | [`references/pc_games.md`](./references/pc_games.md) | Steam integration, desktop performance, shaders |
| **2D Systems** | [`references/2d_games.md`](./references/2d_games.md) | Sprites, tilemaps, 2D physics, pixel art |
| **3D Systems** | [`references/3d_games.md`](./references/3d_games.md) | Shaders, meshes, spatial lighting, PBR materials |
| **Multiplayer** | [`references/multiplayer.md`](./references/multiplayer.md) | Client prediction, server reconciliation |
| **Game Design & Audio** | [`references/game_design.md`](./references/game_design.md), [`references/game_audio.md`](./references/game_audio.md) | GDD, balance economics, spatial audio |
| **Asset Pipeline Linter** | [`scripts/game_engine_linter.py`](./scripts/game_engine_linter.py) | Engine detection & asset pipeline verification |

---

## 🔄 1. The Universal Game Loop & Timestep

```
1. INPUT  ➔ Sample abstracted action bindings (e.g. "jump", "fire").
2. UPDATE ➔ Advance physics & state at a deterministic fixed timestep (e.g. 50Hz / 20ms).
3. RENDER ➔ Interpolate state between ticks and draw to screen (60 FPS = 16.6ms budget).
```

---

## ⚡ 2. Core Architectural Patterns

- **Object Pooling**: Mandatory for high-frequency spawns (projectiles, particle effects).
- **State Machines**: Use finite state machines (FSM) for character states (Idle ➔ Run ➔ Jump).
- **Action Abstraction**: Map actions (`"interact"`) rather than physical hardware keys (`KeyE`).

---

## 🛠️ Verification & Execution Commands

```powershell
# Inspect game engine manifests
python .agent/skills/game-development/scripts/game_engine_linter.py .

# Run self-test suite
python .agent/skills/game-development/scripts/game_engine_linter.py --test
```