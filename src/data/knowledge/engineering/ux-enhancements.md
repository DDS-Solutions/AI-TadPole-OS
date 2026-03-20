---
name: ux-enhancements
description: Technical overview of Tadpole OS user experience upgrades.
---

# UX Enhancement Protocol

Tadpole OS prioritizes high-fidelity feedback and operational efficiency. The following systems are implemented to maintain a premium, responsive interface.

## 1. Dynamic Hierarchy Animations
The `OrgChart` uses SVG-based connection lines that reactive to agent activity.
- **Neural Pulse:** A CSS animation (`neural-pulse`) applied to connection lines when an agent transitions to `thinking` or `coding` states.
- **Visual Feedback:** Provides immediate awareness of swarm activity without requiring log inspection.

## 2. Terminal Tab-Autocomplete
The `Terminal` component features an intelligent `handleKeyDown` interceptor for the `Tab` key.
- **Context Awareness:** Suggests available slash commands (`/help`, `/swarm`, etc.) and active agent names.
- **Fluidity:** Allows rapid command issuing for power users.

## 3. Full-Text Search Indexing
The `Docs` page implements a relevance-weighted search engine.
- **Body Indexing:** Searches not just titles, but the full markdown content of all knowledge base files.
- **Weighted Ranking:** Keyword hits in titles are weighted higher than body hits, ensuring the most relevant results appear first.

## 4. UI/UX Consistency Standards
- **Borders & Glows:** All cards use `zinc-800` borders with contextual glows based on department (e.g., cyan for Operations, purple for Engineering).
- **Glassmorphism:** Heavy use of `backdrop-blur` and low-opacity backgrounds (`bg-zinc-900/50`) to create a layered, "OS" feel.
28: 
29: ## 5. Agent Card Resource & Mission Display
30: The `HierarchyNode` (Agent Card) is optimized for high-density command awareness.
31: - **Hierarchical Metrics:** A dedicated Resource Row displays USD cost, token consumption, and the **"Skills & Workflows"** director.
32: - **Dynamic Scrollable Badges:** Mission objectives are rendered in rectangular, scrollable badges (`rounded-md`, `max-h-[60px]`) to ensure complex mission instructions remain accessible without expanding card dimensions.
33: - **Director Protocol:** The Zap icon label has been standardized to **"Skills & Workflows"** to accurately represent the agent's capability and directive set.
34: - **Z-Index Intelligence:** Dropdown logic dynamically elevates the Resource Row (`z-50`) during interaction to prevent overlap by adjacent nodes or lower UI layers.
