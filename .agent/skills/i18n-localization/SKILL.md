---
name: i18n-localization
description: Internationalization and localization patterns. Detecting hardcoded strings, managing translations, locale files, RTL support.
when_to_use: "When internationalizing an app, managing translations, detecting hardcoded strings, or adding RTL support."
allowed-tools: Read, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / i18n-localization
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Internationalization (i18n) & Localization (L10n) Protocol

> **Purpose**: Decouple UI text into translatable namespace keys and ensure RTL layout compatibility.
> **Workflow Binding**: Used directly during [`/enhance`](../../workflows/enhance.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** i18n rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/translation_frameworks_and_rtl.md`](./references/translation_frameworks_and_rtl.md) | ICU message syntax, CSS logical properties, Intl currency/date formats | Adding languages & fixing RTL layout shifts |

---

## 🌐 1. Core Translation Rules

1. **No Hardcoded UI Strings**: Wrap all user-visible strings in translation keys (`t('auth.login_title')`).
2. **Namespaced JSON Locales**: Organize strings by feature domain (`locales/en/auth.json`, `locales/en/common.json`).
3. **No String Concatenation**: Never concatenate translated strings; use interpolation tokens (`t('welcome', { name })`).
4. **Use CSS Logical Properties**: Always use `margin-inline-start` instead of `margin-left` for seamless RTL support.

---

## 🛠️ 2. Verification Gate

```powershell
# Scan codebase for hardcoded non-internationalized strings
python .agent/skills/i18n-localization/scripts/i18n_checker.py src/
```