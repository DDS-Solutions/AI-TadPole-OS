> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / i18n-localization
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing translation keys or broken RTL layouts.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[I18N_LOCALIZATION]`)

# Internationalization Frameworks & RTL Layouts (L3)

---

## 1. ICU Message Format & Pluralization

```json
{
  "notifications": {
    "unread_count": "{count, plural, =0 {No unread messages} one {# unread message} other {# unread messages}}"
  }
}
```

---

## 2. CSS Logical Properties for RTL Support

```css
/* Always use logical properties instead of physical left/right */
.card-header {
  padding-inline-start: 1.5rem; /* instead of padding-left */
  padding-inline-end: 1.5rem;   /* instead of padding-right */
  border-inline-start: 2px solid var(--color-emerald-accent);
}
```

---

## 3. Native Intl Formatters

```typescript
// Date Formatting
const formattedDate = new Intl.DateTimeFormat(locale, { dateStyle: 'medium' }).format(new Date());

// Currency Formatting
const formattedCost = new Intl.NumberFormat(locale, { style: 'currency', currency: 'USD' }).format(12.5);
```