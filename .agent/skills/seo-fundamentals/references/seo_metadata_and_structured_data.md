# SEO Metadata, OpenGraph & Structured Data Protocols

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / seo-fundamentals / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Mandatory HTML Meta Tags

Every public route must define distinct meta headers:

```html
<!-- Primary Title & Description -->
<title>Product Dashboard | Tadpole OS Sovereign Platform</title>
<meta name="description" content="Manage AI agent swarms and monitor real-time telemetry with Sovereign governance." />
<link rel="canonical" href="https://example.com/dashboard" />

<!-- OpenGraph / Social Media -->
<meta property="og:type" content="website" />
<meta property="og:title" content="Product Dashboard | Tadpole OS" />
<meta property="og:description" content="Manage AI agent swarms and monitor real-time telemetry." />
<meta property="og:image" content="https://example.com/assets/og-cover.png" />
<meta property="og:url" content="https://example.com/dashboard" />

<!-- Twitter Card -->
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="Product Dashboard | Tadpole OS" />
<meta name="twitter:description" content="Manage AI agent swarms and monitor real-time telemetry." />
```

---

## 2. JSON-LD Schema.org Structured Data

Inject JSON-LD for rich snippet rendering:

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Tadpole OS",
  "operatingSystem": "Cross-platform",
  "applicationCategory": "DeveloperApplication",
  "offers": {
    "@type": "Offer",
    "price": "0",
    "priceCurrency": "USD"
  }
}
</script>
```

---

## 3. Robots.txt and Sitemap Protocol
- **`robots.txt`**: Permit public indexing of documentation, restrict internal `/api/` or admin routes.
- **`sitemap.xml`**: Auto-generated XML sitemap updated on deployment containing lastmod dates and priorities.
