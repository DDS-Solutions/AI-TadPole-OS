---
name: game-development
description: Game development orchestrator. Routes to platform-specific skills based on project needs.
when_to_use: "When building games with Unity, Godot, Unreal, Phaser, or any game engine. Routes to platform-specific sub-skills."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / game-development
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Game Development Orchestrator

> **Role**: Core game architecture and routing hub for platform-specific and genre-specific game sub-skills.

---

## 🎯 Progressive Disclosure & Sub-Skill Directory

Read **REQUIRED** core loop rules below; load specialized **Sub-Skills** on demand:

| Domain / Target | Sub-Skill Directory | Focus / Technologies |
|---|---|---|
| **Web Games** | [`web-games/SKILL.md`](./web-games/SKILL.md) | HTML5, Canvas, WebGL, PixiJS, Three.js, Phaser |
| **Mobile Games** | [`mobile-games/SKILL.md`](./mobile-games/SKILL.md) | Touch controls, battery optimization, iOS/Android |
| **PC Games** | [`pc-games/SKILL.md`](./pc-games/SKILL.md) | Steam integration, high-poly assets, desktop performance |
| **2D Systems** | [`2d-games/SKILL.md`](./2d-games/SKILL.md) | Sprites, tilemaps, 2D physics, pixel art |
| **3D Systems** | [`3d-games/SKILL.md`](./3d-games/SKILL.md) | Shaders, meshes, spatial lighting, PBR materials |
| **Multiplayer** | [`multiplayer/SKILL.md`](./multiplayer/SKILL.md) | Client prediction, server reconciliation, lag compensation |
| **Game Design & Audio** | [`game-design/SKILL.md`](./game-design/SKILL.md), [`game-audio/SKILL.md`](./game-audio/SKILL.md) | GDD, balance economics, spatial audio, adaptive sound |

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