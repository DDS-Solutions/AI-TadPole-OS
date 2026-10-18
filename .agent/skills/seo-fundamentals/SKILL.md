---
name: seo-fundamentals
description: "SEO fundamentals, E-E-A-T, Core Web Vitals, metadata schemas, OpenGraph tags, and Google algorithm principles."
when_to_use: "When optimizing web pages for search engines, implementing meta tags, improving E-E-A-T, auditing heading hierarchies, or validating Core Web Vitals."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / seo-fundamentals
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing canonical URLs, duplicate H1 headers, absent OpenGraph cards, or unindexed pages.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# SEO Fundamentals & Semantic Web Standards

> **Philosophy**: Built-in accessibility and semantic HTML are the bedrock of modern SEO. Search visibility requires technical correctness, E-E-A-T authority, and fast Core Web Vitals.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md), [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md), and [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/seo_metadata_and_structured_data.md`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/references/seo_metadata_and_structured_data.md) | JSON-LD schema definitions, OpenGraph card tags, robots.txt, and XML sitemaps | Implementing site metadata & schema markup |
| [`scripts/seo_checker.py`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/scripts/seo_checker.py) | Automated static auditor for title tags, meta descriptions, image alts, and single H1 tags | Verifying public web pages and route templates |

---

## 📐 1. Technical SEO Core Rules

1. **Strict Heading Hierarchy**: Exactly **one `<h1>`** per page. Follow natural descending order (`h1 ➔ h2 ➔ h3`). Never skip levels for styling.
2. **Mandatory Meta Headers**: Every page requires unique `<title>`, `<meta name="description">`, and `<link rel="canonical">`.
3. **Image Alt Attributes**: Every `<img>` tag must include a descriptive, meaningful `alt` attribute (decorative images must use `alt="" aria-hidden="true"`).
4. **Structured Data**: Inject JSON-LD Schema.org blocks for SoftwareApplication, Article, Organization, or FAQ pages.

---

## 📊 2. E-E-A-T & Core Web Vitals Matrix

| Principle | Technical Signals | Performance Floor |
|---|---|---|
| **Experience & Expertise** | Author bylines, published dates, real screenshots | — |
| **Authoritativeness** | Canonical URLs, semantic markup, sitemap linkage | — |
| **Trustworthiness** | HTTPS, security policy headers, privacy disclosures | — |
| **LCP (Largest Contentful Paint)** | Preloaded hero image, optimized web fonts | `< 2.5s` |
| **INP (Interaction to Next Paint)** | Non-blocking event loop, memoized listeners | `< 200ms` |
| **CLS (Cumulative Layout Shift)** | Explicit image dimensions, reserved aspect ratios | `< 0.1` |

---

## 🧠 3. Socratic Gate for SEO Optimization

Before launching or publishing web pages, verify:
- **Gate 1 (Title & Description)**: Does the page have a descriptive, unique `<title>` (50–60 chars) and meta description (120–160 chars)?
- **Gate 2 (Single H1 Tag)**: Does the page contain exactly one `<h1>` element?
- **Gate 3 (Image Alt Attributes)**: Do all images provide descriptive text for screen readers and search crawlers?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Multiple `<h1>` Elements** | Confuses search crawler topic indexing. | Enforce single `<h1>` per page template. |
| **Missing Image `alt`** | Hurts accessibility score and image search ranking. | Add descriptive `alt="Brief explanation"`. |
| **Missing Canonical Tag** | Content scraped or accessed via query parameters creates duplicates. | Set `<link rel="canonical" href="...">`. |
| **Generic Page Titles** | "Dashboard" or "Home" lacks search differentiation. | Use branded pattern: "Feature Name \| Sovereign Platform". |

---

## 🛠️ 5. Deterministic Verification Gate

Audit web routes and pages for SEO compliance:

```powershell
# 1. Run built-in self-tests
python .agent/skills/seo-fundamentals/scripts/seo_checker.py --test

# 2. Audit a web frontend project directory
python .agent/skills/seo-fundamentals/scripts/seo_checker.py src/pages
```