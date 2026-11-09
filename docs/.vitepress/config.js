/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: Documentation System / config
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
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
  srcExclude: ['wiki/**'],
  
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
      "softwareVersion": "1.1.464",
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
      'AI_INTERACTIVE_SETUP.html',
      'STARTER_KITS.html',
      'SMB_LIVE_OPERATION_GUIDE.html',
      'ARCHITECTURE.html',
      'ARCHITECTURE_OVERVIEW.html',
      'SWARM_ORCHESTRATION.html',
      'Knowledge_Memory.html',
      'BLUEPRINT-mission-flight-recorder.html',
      'BLUEPRINT-durable-state-backups.html',
      'BLUEPRINT-universal-macos-build.html',
      'DESIGN_SYNERGY.html',
      'SECURITY.html',
      'Security_Model.html',
      'GOVERNANCE_ROLE_GUIDE.html',
      'ADG_V2.html',
      'OPERATIONS_MANUAL.html',
      'DEPLOYMENT_GUIDE.html',
      'RELEASE_PROCESS.html',
      'TROUBLESHOOTING.html',
      'Agent_Runner_Workflow.html',
      'API_CONTRACT.html',
      'API_REFERENCE.html',
      'CLI_TOOLS.html',
      'CODEBASE_MAP.html',
      'QWEN_LOCAL_INTEGRATION.html',
      'WEBSOCKET_EVENTS.html',
      'TEST_MISSIONS.html',
      'tactical_swarm_mission.html',
      'token_telemetry_mission.html',
      'GLOSSARY.html',
      'specs/mock-smb-digital-twin.html',
      'specs/mission_flight_recorder.html',
      'assessments/dual-purpose-value-20261023.html'
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
      { text: 'Flight Recorder', link: '/BLUEPRINT-mission-flight-recorder' },
      { text: 'SMB Practice', link: '/specs/mock-smb-digital-twin' },
      { text: 'API & Specs', link: '/API_REFERENCE' },
      { text: 'Wiki', link: 'https://github.com/DDS-Solutions/AI-TadPole-OS/wiki' },
      { text: 'Marketing', link: 'https://dds-solutions.github.io/AI-Tadpole-OS-Marketing/' },
      { text: 'GitHub', link: 'https://github.com/DDS-Solutions/AI-Tadpole-OS' }
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Introduction & Quickstart', link: '/GETTING_STARTED' },
          { text: 'Desktop App Installation', link: '/GETTING_STARTED#desktop-app-fastest-start' },
          { text: 'AI Interactive Setup', link: '/AI_INTERACTIVE_SETUP' },
          { text: 'Business Starter Kits', link: '/STARTER_KITS' },
          { text: 'SMB Live Operations Guide', link: '/SMB_LIVE_OPERATION_GUIDE' },
          { text: 'Test Missions Execution', link: '/TEST_MISSIONS' },
        ]
      },
      {
        text: 'Architecture & Swarms',
        items: [
          { text: 'Core Sovereign Engine', link: '/ARCHITECTURE' },
          { text: 'Architecture Overview', link: '/Architecture_Overview' },
          { text: 'Swarm Orchestration', link: '/SWARM_ORCHESTRATION' },
          { text: 'Neural Memory & LanceDB RAG', link: '/Knowledge_Memory' },
          { text: 'Design Synergy & Theme Tokens', link: '/DESIGN_SYNERGY' },
          { text: 'Durable State Backups', link: '/BLUEPRINT-durable-state-backups' },
          { text: 'Universal macOS Build', link: '/BLUEPRINT-universal-macos-build' },
        ]
      },
      {
        text: 'Observability & Flight Recorder',
        items: [
          { text: 'Mission Flight Recorder Blueprint', link: '/BLUEPRINT-mission-flight-recorder' },
          { text: 'Flight Recorder Specification', link: '/specs/mission_flight_recorder' },
          { text: 'Real-time WebSocket Events', link: '/WEBSOCKET_EVENTS' },
          { text: 'Tactical Swarm Mission Guide', link: '/tactical_swarm_mission' },
          { text: 'Token & Telemetry Stream', link: '/token_telemetry_mission' },
        ]
      },
      {
        text: 'SMB Digital Twins & Benchmarks',
        items: [
          { text: 'Mock SMB Digital Twin Spec', link: '/specs/mock-smb-digital-twin' },
          { text: 'Dual-Purpose Value Assessment', link: '/assessments/dual-purpose-value-20261023' },
          { text: 'Mission Benchmarks Standard', link: '/MISSION_BENCHMARKS' },
        ]
      },
      {
        text: 'Security & Sovereign Governance',
        items: [
          { text: 'Zero-Trust Security Framework', link: '/SECURITY' },
          { text: 'Security Model & Guardrails', link: '/Security_Model' },
          { text: 'Dual-Plane Governance Guide', link: '/GOVERNANCE_ROLE_GUIDE' },
          { text: 'Active Documentation Guard (ADG v2)', link: '/ADG_V2' },
        ]
      },
      {
        text: 'Operations & Runbooks',
        items: [
          { text: 'Operations Manual', link: '/OPERATIONS_MANUAL' },
          { text: 'Production Deployment Guide', link: '/DEPLOYMENT_GUIDE' },
          { text: 'Release Governance Process', link: '/RELEASE_PROCESS' },
          { text: 'Troubleshooting & Diagnostics', link: '/TROUBLESHOOTING' },
        ]
      },
      {
        text: 'Developer & Interface Contracts',
        items: [
          { text: 'Agent Runner Workflow', link: '/Agent_Runner_Workflow' },
          { text: 'API Contract & Schemas', link: '/API_CONTRACT' },
          { text: 'REST API Reference', link: '/API_REFERENCE' },
          { text: 'CLI Management Tools', link: '/CLI_TOOLS' },
          { text: 'Codebase Subsystem Map', link: '/CODEBASE_MAP' },
          { text: 'Qwen & Local LLM Integration', link: '/QWEN_LOCAL_INTEGRATION' },
          { text: 'Technical Glossary', link: '/GLOSSARY' },
        ]
      },
      {
        text: 'Sovereign Wiki',
        items: [
          { text: 'Wiki Home', link: 'https://github.com/DDS-Solutions/AI-TadPole-OS/wiki' },
          { text: 'Technical Glossary', link: 'https://github.com/DDS-Solutions/AI-TadPole-OS/wiki/Glossary' },
          { text: 'Developer Appendix', link: 'https://github.com/DDS-Solutions/AI-TadPole-OS/wiki/Developer-Appendix' }
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
