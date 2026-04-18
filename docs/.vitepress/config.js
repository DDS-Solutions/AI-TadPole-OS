import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "Tadpole OS",
  description: "The high-performance, local-first runtime for sovereign multi-agent swarms.",
  base: '/AI-TadPole-OS/', // Set to your public repo name
  ignoreDeadLinks: true,
  vite: {
    build: {
      target: 'esnext'
    }
  },
  
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    // Placeholder for GSC Key - I will update this if the key is provided
    ['meta', { name: 'google-site-verification', content: 'google63801b77e053c16e' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Tadpole OS Documentation' }],
    ['meta', { property: 'og:description', content: 'Explore the technical architecture and orchestration protocols of Tadpole OS.' }],
  ],

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
