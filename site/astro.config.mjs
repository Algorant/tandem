import fs from 'node:fs';

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { ExpressiveCodeTheme } from 'astro-expressive-code';

const verdigrisLight = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/verdigris-light.jsonc', import.meta.url), 'utf-8')
);
const verdigrisDark = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/verdigris-dark.jsonc', import.meta.url), 'utf-8')
);

export default defineConfig({
  site: 'https://trytandem.dev',
  base: '/',
  integrations: [
    starlight({
      title: 'Tandem',
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/Algorant/tandem',
        },
      ],
      customCss: ['./src/styles/verdigris.css'],
      expressiveCode: {
        themes: [verdigrisDark, verdigrisLight],
        useStarlightUiThemeColors: true,
        emitExternalStylesheet: false,
      },
      sidebar: [
        {
          label: 'Docs',
          items: [
            { label: 'Overview', link: '/' },
            { label: 'Concepts', link: '/concepts/' },
            { label: 'Protocol', link: '/protocol/' },
            { label: 'CLI', link: '/cli/' },
            { label: 'TUI', link: '/tui/' },
            { label: 'Extensions', link: '/extensions/' },
            {
              label: 'Guides',
              items: [
                { label: 'Guides overview', link: '/guides/' },
                { label: 'Docs site workflow', link: '/guides/docs-site/' },
                { label: 'Theme tester', link: '/guides/theme-tester/' },
              ],
            },
            { label: 'Reference', link: '/reference/' },
          ],
        },
      ],
    }),
  ],
});
