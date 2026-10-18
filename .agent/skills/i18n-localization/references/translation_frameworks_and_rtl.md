# Internationalization Frameworks & RTL Layouts (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / i18n-localization / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing translation keys, string concatenation, or broken RTL layouts.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[I18N_LOCALIZATION]`)

---

## 1. ICU Message Format & Pluralization

Never perform string concatenation or inline string interpolation for pluralized text. Different languages have distinct pluralization rules (e.g. Slavic languages have distinct forms for 1, 2-4, and 5+). Always use ICU Message syntax:

```json
{
  "notifications": {
    "unread_count": "{count, plural, =0 {No unread messages} one {# unread message} other {# unread messages}}"
  },
  "tasks": {
    "summary": "{gender, select, male {He completed} female {She completed} other {They completed}} {count, plural, one {# task} other {# tasks}}."
  }
}
```

### Usage Pattern in React / TSX:
```tsx
import { useTranslation } from 'react-i18next';

export const UnreadBadge = ({ count }: { count: number }) => {
  const { t } = useTranslation('notifications');
  return <span>{t('unread_count', { count })}</span>;
};
```

---

## 2. CSS Logical Properties Dictionary for Bi-Directional (BiDi) & RTL

When supporting Right-to-Left (RTL) locales like Arabic (`ar`) or Hebrew (`he`), physical CSS properties (`left`, `right`) cause inverted, broken layouts. Replace all physical properties with CSS Logical Properties:

| Physical Property (Forbidden in BiDi) | CSS Logical Replacement (Required) | Behavior in LTR | Behavior in RTL |
|---|---|---|---|
| `margin-left` | `margin-inline-start` | Left margin | Right margin |
| `margin-right` | `margin-inline-end` | Right margin | Left margin |
| `padding-left` | `padding-inline-start` | Left padding | Right padding |
| `padding-right` | `padding-inline-end` | Right padding | Left padding |
| `left: 0` | `inset-inline-start: 0` | Positioned at left edge | Positioned at right edge |
| `right: 0` | `inset-inline-end: 0` | Positioned at right edge | Positioned at left edge |
| `text-align: left` | `text-align: start` | Left-aligned text | Right-aligned text |
| `text-align: right` | `text-align: end` | Right-aligned text | Left-aligned text |
| `border-left` | `border-inline-start` | Left border | Right border |
| `border-right` | `border-inline-end` | Right border | Left border |
| `border-top-left-radius` | `border-start-start-radius` | Top-left corner | Top-right corner |
| `border-top-right-radius` | `border-start-end-radius` | Top-right corner | Top-left corner |

### Handling Directional Icons:
Navigation icons (arrows, chevrons, back buttons) must be mirrored in RTL mode:

```css
[dir="rtl"] .icon-directional {
  transform: scaleX(-1);
}
```
*Note: Media player controls (play, pause, fast-forward), clocks, and progress bars should NOT be mirrored.*

---

## 3. Native JavaScript `Intl` Formatters

Avoid pulling heavy date/currency libraries. Modern JavaScript engines provide high-performance, deterministic localization via the standard `Intl` namespace:

### Date & Time Formatting:
```typescript
// Format: "Oct 17, 2026" in en-US, "17 oct. 2026" in fr-FR
const dateFormatter = new Intl.DateTimeFormat(locale, {
  dateStyle: 'medium',
  timeZone: 'UTC'
});
const formattedDate = dateFormatter.format(new Date());
```

### Currency & Number Formatting:
```typescript
// Format: "$1,250.50" in en-US, "1 250,50 €" in fr-FR
const currencyFormatter = new Intl.NumberFormat(locale, {
  style: 'currency',
  currency: 'USD',
  maximumFractionDigits: 2
});
const formattedAmount = currencyFormatter.format(1250.50);
```

### Relative Time Formatting (e.g. "3 minutes ago"):
```typescript
const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
const relativeStr = rtf.format(-3, 'minute'); // "3 minutes ago" or "il y a 3 minutes"
```

---

## 4. Pseudo-Localization & Visual Stress Testing

To verify that UI components tolerate text expansion (German/Russian typically expand 30–50% compared to English) and have no hardcoded strings, use **pseudo-localization**:

1. **Text Expansion**: Append brackets and elongation tokens:
   - English: `Save Settings`
   - Pseudo: `[!!! Ṩàṽé Ṩéţţîñğş !!! ~~~~~~~~~]`
2. **Layout Verification**:
   - Check if buttons truncate, wrap awkwardly, or overflow containers.
   - Any raw Latin text that lacks accents immediately identifies a leaked hardcoded string.

---

## 5. Verification Tools

Audit locale keys and detect hardcoded strings using the skill's validation script:
- [`scripts/i18n_checker.py`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/scripts/i18n_checker.py)