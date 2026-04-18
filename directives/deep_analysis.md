# 🔬 Directive: Deep Analysis (SOP-STR-05)

## 🎯 Primary Objective
Deliver exhaustive, multi-dimensional intelligence on complex technical or strategic problems. This directive ensures that "Deep Dive" missions go beyond surface-level summaries to identify hidden patterns, edge cases, and systemic implications.

---

## 🏗️ Analysis Phases

### 1. Information Ingestion (The Net)
- **Action**: Research primary sources (codebase, logs, external docs) and secondary sources (market trends, analyst reports).
- **Tool**: Use `search_web` and `view_file` on the top 10 relevant files/URLs.

### 2. Contradiction Mapping
- **Goal**: Identify "Logical Gaps" where documentation contradicts implementation or where multiple sources provide conflicting data.
- **Action**: Use the `aletheia-reasoning` skill to cross-examine findings.

### 3. Synthesis (The Core)
- **Focus**: Identify the "Leverage Point" — the one change or insight that has the highest impact on the problem.
- **Guidance**: Avoid "Insight-Dilution." Focus on the 3 most significant findings.

---

## 🛠️ Output SOP

### 1. Executive Summary
- **Content**: The "Bottom Line Up Front" (BLUF). 

### 2. Technical Evidence
- **Content**: Code snippets, benchmark tables, and Mermaid dependency graphs.

### 3. Strategic Path
- **Content**: Recommendations for the `feature_roadmap.md` or `sprint_planning.md`.

---

## 📊 Quality Gate
All analyses must be "Stress-Tested" by a specialist agent (e.g., "Critical Reviewer") to ensure zero confirmation bias and absolute technical accuracy.