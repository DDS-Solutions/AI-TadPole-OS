---
name: app-builder
description: Main application building orchestrator. Creates full-stack applications from natural language requests.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, Agent
---

# App Builder

**Build on rock, not sand.**

## Core Templates
- **Web**: `nextjs-fullstack` (App), `nextjs-saas` (Product), `nextjs-static` (Landing).
- **Mobile**: `react-native-app` (Expo), `flutter-app`.
- **Backend**: `express-api` (Node), `python-fastapi`.
- **Desktop**: `electron-desktop`.

---

## 🧠 Aletheia Reasoning Protocol (Scaffolding)

### 1. Generator (Stack Selection)
*   **Context Fit**: "Weekend hack (Vite) vs Enterprise SaaS (Next.js)?".
*   **Ecosystem Scan**: "Is this library maintained?".
*   **Future Proofing**: "Will this lock us into a dying vendor?".

### 2. Verifier (Complexity Audit)
*   **Overkill Check**: "Do we need Kubernetes for a blog?".
*   **Glue Cost**: "How much code just to connect A to B?".
*   **Learning Curve**: "Can the user maintain this?".

### 3. Reviser (Simplification)
*   **Default Safe**: "Enable strict mode. Enable linting.".
*   **Trim Fat**: Remove boilerplate immediately.

---

## 🛡️ Security & Safety Protocol (Builder)

1.  **Secure Defaults**: Headers (Helmet/CSP) enabled by default.
2.  **Dependency Scan**: Run `npm audit` on template.
3.  **Secret Placholders**: Generate `.env.example`, NEVER `.env` with keys.
4.  **Input Sanitization**: Boilerplate includes safe query examples.
