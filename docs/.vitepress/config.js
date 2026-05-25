/**
 * @docs ARCHITECTURE:Documentation
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[config]` in observability traces.
 */

import { defineConfig } from 'vitepress'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export default defineConfig({
  title: "Tadpole OS",
  description: "The high-performance, local-first runtime for sovereign multi-agent swarms.",
  base: '/AI-TadPole-OS/',
  ignoreDeadLinks: true,
  lastUpdated: true,
  
  vite: {
    build: {
      target: 'esnext'
    }
  },
  
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#09090b' }],
    ['meta', { name: 'google-site-verification', content: 'google63801b77e053c16e' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Tadpole OS Documentation' }],
    ['meta', { property: 'og:description', content: 'Explore the technical architecture and orchestration protocols of Tadpole OS.' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=JetBrains+Mono:wght@400;600&display=swap' }]
  ],

  // AI & Crawler Optimization (GEO/SEO)
  transformHead: ({ pageData, siteData }) => {
    const canonicalUrl = `https://dds-solutions.github.io/AI-Tadpole-OS/${pageData.relativePath.replace(/\.md$/, '.html')}`
    
    // JSON-LD Structured Data for AI Agents
    const schema = {
      "@context": "https://schema.org",
      "@type": "SoftwareApplication",
      "name": "Tadpole OS",
      "applicationCategory": "System Software",
      "operatingSystem": "Cross-platform (Rust-based)",
      "description": siteData.description,
      "softwareVersion": "1.1.16",
      "author": {
        "@type": "Organization",
        "name": "DDS Solutions"
      },
      "url": "https://dds-solutions.github.io/AI-Tadpole-OS/"
    }

    return [
      ['link', { rel: 'canonical', href: canonicalUrl }],
      ['meta', { property: 'og:url', content: canonicalUrl }],
      ['script', { type: 'application/ld+json' }, JSON.stringify(schema)],
      // GEO Hint: explicit content summary for AI scanners
      ['meta', { name: 'ai-content-summary', content: pageData.frontmatter.ai_summary || pageData.description || siteData.description }]
    ]
  },

  // Automated Sitemap Generation
  buildEnd: async ({ outDir }) => {
    const pages = [
      '',
      'GETTING_STARTED.html',
      'ARCHITECTURE.html',
      'SWARM_ORCHESTRATION.html',
      'API_REFERENCE.html',
      'GLOSSARY.html',
      'TROUBLESHOOTING.html',
      'TEST_MISSIONS.html',
      'ARCHITECTURE_OVERVIEW.html',
      'CLI_TOOLS.html'
    ]

    const sitemapContent = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  ${pages.map(page => `
  <url>
    <loc>https://dds-solutions.github.io/AI-Tadpole-OS/${page}</loc>
    <lastmod>${new Date().toISOString()}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>${page === '' ? '1.0' : '0.8'}</priority>
  </url>`).join('')}
</urlset>`
    writeFileSync(resolve(outDir, 'sitemap.xml'), sitemapContent)
  },

  themeConfig: {
    logo: '/logo.png',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/GETTING_STARTED' },
      { text: 'Architecture', link: '/ARCHITECTURE' },
      { text: 'Marketing Site', link: 'https://dds-solutions.github.io/AI-Tadpole-OS-Marketing/' },
      { text: 'GitHub', link: 'https://github.com/DDS-Solutions/AI-Tadpole-OS' }
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Introduction', link: '/GETTING_STARTED' },
          { text: 'Installation', link: '/GETTING_STARTED#installation' },
          { text: 'Test Missions', link: '/TEST_MISSIONS' },
        ]
      },
      {
        text: 'Architecture',
        items: [
          { text: 'Core Engine', link: '/ARCHITECTURE' },
          { text: 'Swarm Orchestration', link: '/SWARM_ORCHESTRATION' },
          { text: 'Neural Memory (RAG)', link: '/ARCHITECTURE#memory-layer' },
        ]
      },
      {
        text: 'References',
        items: [
          { text: 'API Reference', link: '/API_REFERENCE' },
          { text: 'Glossary', link: '/GLOSSARY' },
          { text: 'Troubleshooting', link: '/TROUBLESHOOTING' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/DDS-Solutions/AI-Tadpole-OS' }
    ],

    footer: {
      message: 'Sovereign Intelligence Architecture.',
      copyright: 'Copyright © 2024-present DDS Solutions'
    }
  }
})

// Metadata: [config]
