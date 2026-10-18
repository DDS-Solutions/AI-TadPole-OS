---
name: i18n-localization
description: "Internationalization (i18n) and localization (L10n) patterns. Detecting hardcoded strings, managing translations, locale files, RTL support, and Intl formatting."
when_to_use: "When internationalizing an app, adding or updating translations, detecting hardcoded UI strings, implementing BiDi/RTL layouts, or formatting currency/dates."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / i18n-localization
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing translation keys, hardcoded UI strings, or broken RTL layouts.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Internationalization (i18n) & Localization (L10n) Protocol

> **Purpose**: Decouple all user-visible text into translatable namespace keys, eliminate string concatenation bugs, enforce Bi-Directional (BiDi/RTL) CSS logical properties, and standardize deterministic `Intl` formatting.  
> **Workflow Bindings**: Integrated directly into [`/enhance`](file:///D:/TadpoleOS-Dev/.agent/workflows/enhance.md) and [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/translation_frameworks_and_rtl.md`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/references/translation_frameworks_and_rtl.md) | ICU message syntax, CSS logical properties dictionary, Intl currency/date patterns | Adding languages, formatting currencies, & fixing RTL layout shifts |
| [`scripts/i18n_checker.py`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/scripts/i18n_checker.py) | Automated locale completeness auditor and hardcoded string scanner | Verifying translation coverage and preventing hardcoded string leaks |

---

## 🌐 1. The 4 Pillars of Sovereign i18n

### Pillar 1: Zero Hardcoded UI Text
- All user-facing strings must be wrapped in translation keys: `t('auth.login_title')`.
- Organize translations into feature namespaces (`locales/en/auth.json`, `locales/en/common.json`, `locales/en/dashboard.json`).
- Always internationalize accessible attributes: `aria-label={t('common.close')}`, `title={t('settings.edit')}`.

### Pillar 2: ICU Message Syntax Over String Concatenation
- **NEVER concatenate translated strings**:
  ```typescript
  // ❌ FORBIDDEN: Grammar breaks in other languages
  const msg = t('welcome') + " " + user.name + " (" + count + " " + t('items') + ")";

  // ✅ SOVEREIGN: ICU interpolation with full grammatical context
  const msg = t('welcome_banner', { name: user.name, count });
  ```
- Use ICU plurals (`zero`, `one`, `other`) to support varied language grammar without fragile code conditionals.

### Pillar 3: CSS Logical Properties for Seamless BiDi / RTL
- Never use physical directional properties (`margin-left`, `padding-right`, `left: 0`, `text-align: left`).
- Use CSS logical counterparts (`margin-inline-start`, `padding-inline-end`, `inset-inline-start: 0`, `text-align: start`).
- Mirror directional navigation arrows (`transform: scaleX(-1)` in `[dir="rtl"]`), but preserve media controls, clocks, and progress bars.

### Pillar 4: Deterministic Native `Intl` Formatting
- Never manually format dates (e.g. `MM/DD/YYYY`) or currencies (e.g. `"$ " + amount`).
- Use native `Intl.DateTimeFormat(locale, options)` and `Intl.NumberFormat(locale, { style: 'currency', currency })`.

---

## 🧠 2. Socratic Gate for i18n Alignment

Before declaring an internationalization task complete, verify these 4 gates:
1. **Key Namespace Hygiene**: Are new translation keys nested under the appropriate domain namespace rather than a giant flat file?
2. **Grammar & Concatenation Check**: Are there any string concatenations or template literals combining translated text fragments?
3. **BiDi / RTL Flipping**: Does the layout maintain symmetry and alignment when tested with `dir="rtl"`?
4. **Number & Date Formats**: Are timestamps, percentages, and currencies formatted using standard `Intl` APIs?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Hardcoded UI Strings** | Raw text `<button>Save</button>` fails in non-English locales. | Wrap with `t('common.save')` and register in base locale JSON. |
| **String Concatenation** | `t('deleted') + ' ' + count + ' ' + t('files')` produces ungrammatical translations. | Use ICU message syntax: `{count, plural, one {# file deleted} other {# files deleted}}`. |
| **Physical CSS Directionals** | `padding-left: 24px` causes misaligned content in RTL mode. | Replace with `padding-inline-start: 24px`. |
| **Hardcoded Formats** | Formatting currency as `'$' + val` breaks in EU/UK/Japan. | Use `new Intl.NumberFormat(locale, { style: 'currency', currency }).format(val)`. |
| **Locale Key Drift** | Adding a key to `en.json` but omitting it from `es.json` or `fr.json`. | Run `python .agent/skills/i18n-localization/scripts/i18n_checker.py` to catch missing keys. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate i18n integrity and locale coverage:

```powershell
# 1. Run built-in self tests
python .agent/skills/i18n-localization/scripts/i18n_checker.py --test

# 2. Audit codebase for hardcoded strings and missing locale keys
python .agent/skills/i18n-localization/scripts/i18n_checker.py src/
```