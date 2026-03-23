---
name: frontend-design
description: Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Frontend Design System

**Philosophy**: Every pixel has purpose. Restraint is luxury.

## 🛑 Modern SaaS "Safe Harbor" (ANTI-PATTERNS)
**Avoid these defaults:**
- **Bento Grids**: Cliché.
- **Hero Split**: Boring.
- **Mesh Gradients**: Lazy.
- **Glassmorphism**: Overused.
- **Deep Cyan / Fintech Blue**: Safe.

**Instead:** Take risks. Use massive type, asymmetry, and radical color.

---

## 🧠 Aletheia Reasoning Protocol (Visual Design)

**Beauty is a function of utility.**

### 1. Generator (Concept Divergence)
*   **Concept 1 (Safe)**: "clean, standard, expected".
*   **Concept 2 (Bold)**: "high contrast, large type, unconventional grid".
*   **Concept 3 (Weird)**: "brutalist, raw, experimental".

### 2. Verifier (Accessibility Audit)
*   **Contrast**: "Is this grey text readable on this grey background?".
*   **Touch**: "Is this button big enough for a thumb?".
*   **Motion**: "Will this parallax effect make someone sick?".

### 3. Reviser (Reduction)
*   **Palette**: "Remove 2 colors. We have too many.".
*   **Noise**: "Remove that border. Use space instead.".

---

## 🛡️ Security & Safety Protocol (Frontend)

1.  **XSS**: Never use `dangerouslySetInnerHTML` without sanitization.
2.  **Asset Safety**: Don't load scripts from untrusted CDNs.
3.  **Clickjacking**: Ensure UI cannot be framed transparently.
4.  **Storage**: Don't store JWTs in LocalStorage.

## Design Principles
- **Constraint Analysis**: Timeout? Content? Brand? Tech? Audience?
- **UX Psychology**: Hick's Law, Fitts' Law, Miller's Law.
- **Layout**: Golden Ratio, 8-Point Grid.
- **Color**: 60-30-10 Rule. Emotion-driven.
- **Type**: Paired scales. 45-75 chars/line.
- **Motion**: Purposeful. Easing matched to context.
