---
name: geo-fundamentals
description: Generative Engine Optimization execution protocol. Audits and injects structured schemas, statistical citation magnets, and E-E-A-T entity markers.
when_to_use: "MANDATORY during the SHIP phase for any public-facing web content, landing pages, blog posts, or documentation. Use when the goal is to be cited as a source by AI engines."
allowed-tools: Read, Write, Glob, Grep, Execute
version: "2.1.0"
priority: medium
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / geo-fundamentals
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: AI-invisible content, "ghost" pages (no citations), or outdated metadata leading to LLM hallucinations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# 🌍 Generative Engine Optimization (GEO) Protocol

> **Purpose**: Ensure all public-facing web content, technical documentation, and landing pages are discoverable, citable, and trusted by AI search engines (Perplexity, ChatGPT, Gemini, Claude).
> **Workflow Binding**: Used during [`/deploy`](../../workflows/deploy.md), [`/create`](../../workflows/create.md), and the `SHIP` behavioral mode.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core workflow below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/geo_citation_magnets.md`](./references/geo_citation_magnets.md) | Production templates for JSON-LD schemas, statistical formulas, and FAQ definitions | Writing or refactoring public pages for AI engine discoverability |
| [`scripts/geo_checker.py`](./scripts/geo_checker.py) | Component-aware AI citation readiness scanner and audit engine | Automated page audit or `--test` validation |

---

## 🔄 The GEO Operational Loop

You MUST follow this sequence for every public page:

### 1. The Audit (`geo_checker.py`)
Before claiming content is citation-ready, run the verification tool:
```powershell
python .agent/skills/geo-fundamentals/scripts/geo_checker.py src/pages
```

**Score Thresholds:**
- **Score $< 60\%$**: 🔴 Critical Failure. Content lacks structured data and is invisible to AI scrapers.
- **Score $60-80\%$**: 🟡 Sub-optimal. Missing key citation magnets or direct answer definitions.
- **Score $> 80\%$**: 🟢 Sovereign Standard. High probability of AI citation and indexing.

### 2. The Injection Phase
Based on the issues reported, inject structured elements:

| Missing Element | Implementation Action |
|---|---|
| **JSON-LD** | Inject `<script type="application/ld+json">` with `@type: TechArticle` or `FAQPage`. |
| **Direct Answers** | Add explicit `"In short, [Concept] is [Definition]"` sentences at the start of sections. |
| **Citation Magnets** | Add verified percentages, benchmarks (e.g. `"74.2% reduction"`), or expert quotes. |
| **Entity Signals** | Include author credentials, ISO 8601 timestamps (`dateModified`), and clear H1-H3 trees. |
| **Structured Data** | Convert descriptive paragraphs into comparative tables or structured lists. |

### 3. Verification Gate
Re-run `geo_checker.py`. Pages must achieve $\ge 80\%$ before shipping to production.

---

## 🛡️ Socratic Gate: Citation Readiness Audits

Before finalizing public documentation or landing pages, confirm:
1. **Machine Parseability**: Is core entity data available in raw JSON-LD without client JavaScript execution?
2. **First-Paragraph Clarity**: Does the opening paragraph answer the query intent in $\le 2$ declarative sentences?
3. **Evidence Density**: Are claims backed by verifiable internal numbers, benchmarks, or authoritative citations?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Keyword Stuffing | AI penalty and loss of semantic trust | Focus on semantic entities and hierarchical ontology |
| Wall of Text | AI extractors skip dense unstructured blocks | Break into tables, bullet lists, and summary cards |
| Vague Timestamps ("recently", "2 years ago") | LLM hallucination of temporal staleness | Use ISO 8601 timestamps (`2026-10-17T00:00:00Z`) |
| Unnamed / Ghost Authors | Low E-E-A-T score | Bind author to `Organization` or verified `Person` schema |

---

## 🛠️ Verification & Execution Commands

```powershell
# Scan page directory for GEO readiness
python .agent/skills/geo-fundamentals/scripts/geo_checker.py src/pages

# Run self-test suite
python .agent/skills/geo-fundamentals/scripts/geo_checker.py --test
```
