import fs from 'node:fs';

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { ExpressiveCodeTheme } from 'astro-expressive-code';

const siteTitle = 'Tandem';
const siteDescription = 'Local-first coordination for humans and agents working in the same repository.';
const siteUrl = 'https://trytandem.dev';
const socialImage = `${siteUrl}/social-card.svg`;

const verdigrisLight = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/verdigris-light.jsonc', import.meta.url), 'utf-8')
);
const verdigrisDark = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/verdigris-dark.jsonc', import.meta.url), 'utf-8')
);

export default defineConfig({
  site: siteUrl,
  base: '/',
  redirects: {
    '/workspace/': '/concepts/#board-storage-and-sync',
    '/protocol/': '/concepts/#board-storage-and-sync',
    '/guides/upgrading-to-independent-sync/': '/concepts/#board-storage-and-sync',
    '/web/': '/tui/#read-only-web-view',
    '/extensions/': '/agents/',
    '/skills/': '/agents/',
    '/guides/agents-and-adapters/': '/agents/',
    '/guides/decisions/': '/concepts/',
    '/guides/human-in-the-loop/': '/concepts/',
    '/guides/fully-agentic-workflow/': '/concepts/',
    '/reference/': '/cli/',
    '/guides/': '/concepts/',
  },
  integrations: [
    starlight({
      title: siteTitle,
      description: siteDescription,
      tagline: 'Plain Markdown tasks, accords, decisions, validation, and logs.',
      titleDelimiter: '·',
      favicon: '/favicon.svg',
      logo: {
        dark: './src/assets/tandem-mark.svg',
        light: './src/assets/tandem-mark-light.svg',
        alt: 'Tandem linked work mark',
      },
      social: [
        {
          icon: 'github',
          label: 'Tandem on GitHub',
          href: 'https://github.com/Algorant/tandem',
        },
      ],
      head: [
        {
          tag: 'meta',
          attrs: {
            name: 'theme-color',
            content: '#1d2021',
            media: '(prefers-color-scheme: dark)',
          },
        },
        {
          tag: 'meta',
          attrs: {
            name: 'theme-color',
            content: '#fbf1c7',
            media: '(prefers-color-scheme: light)',
          },
        },
        { tag: 'meta', attrs: { property: 'og:type', content: 'website' } },
        { tag: 'meta', attrs: { property: 'og:site_name', content: siteTitle } },
        { tag: 'meta', attrs: { property: 'og:title', content: 'Tandem documentation' } },
        { tag: 'meta', attrs: { property: 'og:description', content: siteDescription } },
        { tag: 'meta', attrs: { property: 'og:url', content: siteUrl } },
        { tag: 'meta', attrs: { property: 'og:image', content: socialImage } },
        {
          tag: 'meta',
          attrs: {
            property: 'og:image:alt',
            content: 'Tandem: local-first coordination for human and agent work.',
          },
        },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'meta', attrs: { name: 'twitter:title', content: 'Tandem documentation' } },
        { tag: 'meta', attrs: { name: 'twitter:description', content: siteDescription } },
        { tag: 'meta', attrs: { name: 'twitter:image', content: socialImage } },
      ],
      customCss: ['./src/styles/verdigris.css'],
      routeMiddleware: ['./src/starlight-route-data.ts'],
      expressiveCode: {
        defaultProps: { frame: 'none' },
        themes: [verdigrisDark, verdigrisLight],
        useStarlightUiThemeColors: true,
        emitExternalStylesheet: false,
      },
      tableOfContents: false,
      pagination: false,
      markdown: { headingLinks: false },
      sidebar: [
        { label: 'Home', link: '/' },
        { label: 'Quickstart', link: '/quick-start/' },
        { label: 'Concepts', link: '/concepts/' },
        { label: 'Agents', link: '/agents/' },
        { label: 'TUI and web', link: '/tui/' },
        {
          label: 'Reference',
          items: [
            { label: 'CLI', link: '/cli/' },
            { label: 'Configuration', link: '/reference/configuration/' },
          ],
        },
      ],
    }),
  ],
});
