import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Wednes Serverless Engine',
  description: 'Ultra-low latency WebAssembly serverless runtime powered by Wasmtime Component Model with HTTP/3 QUIC and zero cold-starts.',
  base: process.env.VITEPRESS_BASE || '/wednesdev-serverless-engine/',
  cleanUrls: true,
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', href: '/wednesdev-serverless-engine/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#3eaf7c' }],
    ['meta', { name: 'og:type', content: 'website' }],
    ['meta', { name: 'og:title', content: 'Wednes Serverless Engine' }],
    ['meta', { name: 'og:description', content: 'High-performance Wasmtime-based serverless runtime with HTTP/3 QUIC' }]
  ],

  themeConfig: {
    logo: '/logo.png',
    siteTitle: 'Wednes Serverless Engine',

    nav: [
      { text: 'Guide', link: '/getting-started/overview', activeMatch: '/getting-started/' },
      { text: 'Core Concepts', link: '/core-concepts/wednes-yaml', activeMatch: '/core-concepts/' },
      { text: 'CLI Reference', link: '/cli-reference/wednesd', activeMatch: '/cli-reference/' },
      { text: 'SDKs', link: '/sdks/rust', activeMatch: '/sdks/' },
      { text: 'Production', link: '/production/docker', activeMatch: '/production/' },
      {
        text: 'v0.3.0',
        items: [
          { text: 'Changelog', link: 'https://github.com/wednesdev-id/wednesdev-serverless-engine/releases' },
          { text: 'Roadmap & Specs', link: '/getting-started/architecture' },
          { text: 'Control Plane UI', link: 'https://cp.wednesdev.id' }
        ]
      }
    ],

    sidebar: [
      {
        text: 'Getting Started',
        collapsed: false,
        items: [
          { text: 'Overview', link: '/getting-started/overview' },
          { text: 'Quickstart', link: '/getting-started/quickstart' },
          { text: 'Architecture', link: '/getting-started/architecture' }
        ]
      },
      {
        text: 'Core Concepts',
        collapsed: false,
        items: [
          { text: 'wednes.yaml Specification', link: '/core-concepts/wednes-yaml' },
          { text: 'Zero-Knowledge Encrypted Artifacts', link: '/core-concepts/zero-knowledge-artifacts' },
          { text: 'HTTP/3 QUIC Protocol', link: '/core-concepts/http3-quic' }
        ]
      },
      {
        text: 'CLI & Daemon Reference',
        collapsed: false,
        items: [
          { text: 'Daemon Overview (wednesd)', link: '/cli-reference/wednesd' },
          { text: 'Commands & Flags', link: '/cli-reference/commands-flags' },
          { text: 'Build & Deploy Workflows', link: '/cli-reference/build-deploy' }
        ]
      },
      {
        text: 'Language SDKs',
        collapsed: false,
        items: [
          { text: 'Rust SDK', link: '/sdks/rust' },
          { text: 'TypeScript / JavaScript SDK', link: '/sdks/typescript' },
          { text: 'Python SDK', link: '/sdks/python' }
        ]
      },
      {
        text: 'Production & Deployment',
        collapsed: false,
        items: [
          { text: 'Docker Deployment', link: '/production/docker' },
          { text: 'systemd Service', link: '/production/systemd' },
          { text: 'Resource Limits & Isolation', link: '/production/resource-limits' },
          { text: 'Benchmarks & Performance', link: '/production/benchmarks' }
        ]
      }
    ],

    search: {
      provider: 'local',
      options: {
        locales: {
          root: {
            translations: {
              button: {
                buttonText: 'Search documentation',
                buttonAriaLabel: 'Search documentation'
              },
              modal: {
                noResultsText: 'No results found',
                resetButtonTitle: 'Reset search',
                footer: {
                  selectText: 'to select',
                  navigateText: 'to navigate',
                  closeText: 'to close'
                }
              }
            }
          }
        }
      }
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/wednesdev-id/wednesdev-serverless-engine' }
    ],

    footer: {
      message: 'Released under Apache-2.0 / MIT License. Open Source Cloud Native Computing.',
      copyright: 'Copyright © 2026 WednesDev Team. All rights reserved.'
    },

    editLink: {
      pattern: 'https://github.com/wednesdev-id/wednesdev-serverless-engine/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})
