>This rewrite transforms the **GEO Fundamentals** document from a passive knowledge base into a **Hardened Execution Protocol**.

It now treats the `geo_checker.py` script as the "Source of Truth." Instead of the AI guessing if the content is optimized, it must now follow a rigorous **Audit $\rightarrow$ Fix $\rightarrow$ Verify** loop. I have also integrated this skill into the `SHIP` behavioral mode, ensuring that no public-facing content is released without an AI-citation readiness check.

***

# Revised SKILL.md

--- File: SKILL.md ---
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: AI-invisible content, "ghost" pages (no citations), or outdated metadata leading to LLM hallucinations.
> - **Telemetry Link**: Search `[GEO_CHECKER]` in audit logs.
>
> ### AI Assist Note
> Generative Engine Optimization (GEO) Protocol. Ensures content is discoverable and citable by LLMs (ChatGPT, Perplexity, Claude).
>
> ### 🔍 Debugging & Observability
> Mandatory verification via `scripts/geo_checker.py`.

---
name: geo-fundamentals
description: Execution protocol for Generative Engine Optimization. Uses a specific audit script to ensure content is highly citable by AI search engines.
when_to_use: "MANDATORY during the `SHIP` phase for any public-facing web content, landing pages, blog posts, or documentation. Use when the goal is to be cited as a source by AI engines."
allowed-tools: Read, Write, Glob, Grep, Execute
version: 2.0
priority: MEDIUM
---

# 🌍 GEO Execution Protocol

GEO (Generative Engine Optimization) is the process of making content "digestible" for RAG (Retrieval-Augmented Generation) systems. Unlike SEO, which targets click-through rates, GEO targets **Citation Rate** and **Entity Authority**.

## ⛓️ Pipeline Integration
This skill is a "Quality Gate" in the `SHIP` behavioral mode:
**`IMPLEMENT`** (Write Content) $\rightarrow$ **`GEO-AUDIT`** (Run Script) $\rightarrow$ **`GEO-FIX`** (Inject Elements) $\rightarrow$ **`SHIP`** (Deploy)

---

## 🔄 The GEO Operational Loop

You MUST follow this sequence for every public page:

### 1. The Audit (`geo_checker.py`)
Before claiming a page is "optimized," run the verification script:
`python scripts/geo_checker.py <project_path>`

**Analyze the output:**
- **Score $< 60\%$**: 🔴 Critical Failure. Content is likely invisible to AI engines.
- **Score $60-80\%$**: 🟡 Sub-optimal. Missing key "citation magnets."
- **Score $> 80\%$**: 🟢 Sovereign Standard. High probability of AI citation.

### 2. The Injection Phase (The "Fix")
Based on the script's `issues` list, inject the following elements into the code:

| Missing Element | Implementation Action |
|-------------------|----------------------|
| **JSON-LD** | Inject a `<script type="application/ld+json">` block with `@type: Article` or `FAQPage`. |
| **Direct Answers** | Add "Simply put, [Concept] is [Definition]" paragraphs at the top of sections. |
| **Citation Magnets**| Inject original stats, percentages (e.g., "85% of users..."), or expert quotes. |
| **Entity Signals** | Add author credentials, "Last Updated" timestamps, and clear H1-H3 hierarchies. |
| **Structured Data** | Convert long paragraphs into comparison tables or bulleted lists. |

### 3. The Verification
Re-run `geo_checker.py`. You cannot proceed to `SHIP` until the `average_score` is $\ge 80\%$.

---

## 🎯 High-Value GEO Patterns

### The "Citation Magnet" Formula
AI engines prioritize content that provides **unique, structured data**. Use these patterns:

- **The Statistical Claim**: *"According to our internal analysis of 500+ repos, [X] leads to [Y]% better performance."*
- **The Comparison Table**: Create a table comparing your solution to 3 competitors.
- **The Definition Box**: Use `<dfn>` tags or bold "is defined as" statements.
- **The Expert Quote**: *"As [Name], [Title], states: '[Quote]'"*

### The Schema Blueprint
Always ensure the following JSON-LD entities are present in the `<head>`:
- `Organization` or `Person` (Establishes E-E-A-T).
- `Article` or `TechArticle` (Defines the content type).
- `FAQPage` (Creates a direct "question-answer" map for the LLM).

---

## 🛠️ Script Integration Guide

### Running the Audit
```bash
# Run the GEO checker on the current project
python scripts/geo_checker.py .
```

### Interpreting Results
The script outputs a JSON block at the end. Use this to determine if the task is complete:
```json
{
  "script": "geo_checker",
  "average_score": 85,
  "passed": true
}
```
**If `passed` is `false`, you MUST identify the `issues` listed in the console output and fix them before concluding the task.**

---

## 🚫 Anti-Patterns (GEO Failures)

| ❌ Don't | ✅ Do |
|----------|-------|
| **Keyword Stuffing** | Use **Semantic Entities** (related concepts, not just words). |
| **Wall of Text** | Use **Lists, Tables, and H2s** (AI prefers structured chunks). |
| **Vague Dates** | Use **ISO 8601 timestamps** (`datePublished`, `dateModified`). |
| **Hidden Authors** | Use **Person Schema** to link content to a real expert. |
| **Generic Summary** | Use a **TL;DR** that contains the primary "answer" to the user's intent. |

---

## 📊 Measurement of Success

A page is considered "Sovereign Optimized" when:
1. `geo_checker.py` returns a score of $\ge 80\%$.
2. The page contains at least one **Comparison Table**.
3. The page contains at least one **Structured FAQ**.
4. The page has a valid **JSON-LD Article/Person** schema.

[//]: # (Metadata: [SKILL])


[//]: # (Metadata: [SKILL])
