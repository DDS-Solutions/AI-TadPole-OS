# Tadpole OS - Mobile Companion App (Android)

> **Design Alignment**: Built in strict adherence to the [Tadpole OS Sovereign Design Contract (`DESIGN.md`)](../../DESIGN.md).

---

## 🎨 Mobile Visual Identity & Design Tokens

The Android companion app carries over the exact design aesthetic, HSL-tinted obsidian dark mode, and color palette from the main desktop dashboard:

| Design Token | Color Hex | Use Case in Mobile App |
|---|---|---|
| **Obsidian Background** | `#090D16` | Main screen background (`background`) |
| **Surface Layer 1** | `#111827` / `#1E293B` | Approval Cards, Health Grid Cards, Card Containers |
| **Emerald Accent** | `#10B981` | "Sign & Approve" button, active/running status indicators |
| **Cyan Accent** | `#06B6D4` | Agent Health panel icons, active telemetry counters |
| **Amber Warning** | `#F59E0B` | Halted agent state, pending approval warning badges |
| **Red Alert** | `#DC2626` / `#EF4444` | Emergency Swarm Panic Freeze Switch, reject buttons |

---

## 🚫 Mobile Anti-Patterns Checklist (from `DESIGN.md`)

1. **No Pure `#000000`**: Always use deep HSL-tinted obsidian (`#090D16`).
2. **No Card Inception**: Avoid nesting cards inside cards with duplicate borders.
3. **Monospace for Technical Metrics**: Use `FontFamily.Monospace` for trace IDs, timestamps, and tool names.
4. **Accessible Contrast**: Ensure text contrast on status badges (`#10B981` text on dark emerald tint background).

---

## 🚀 Opening in Android Studio

1. Launch **Android Studio**.
2. Select `File -> Open` and choose `apps/mobile-android`.
3. Gradle will sync dependencies automatically.
