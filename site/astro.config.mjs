import fs from 'node:fs';

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { ExpressiveCodeTheme } from 'astro-expressive-code';

const gruvboxLight = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/gruvbox-light-medium.jsonc', import.meta.url), 'utf-8')
);
const gruvboxDark = ExpressiveCodeTheme.fromJSONString(
  fs.readFileSync(new URL('./src/styles/shiki/gruvbox-dark-medium.jsonc', import.meta.url), 'utf-8')
);

export default defineConfig({
  site: 'https://algorant.github.io',
  base: '/tandem',
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
      customCss: ['./src/styles/gruvbox.css'],
      expressiveCode: {
        themes: [gruvboxLight, gruvboxDark],
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
              ],
            },
            { label: 'Reference', link: '/reference/' },
          ],
        },
      ],
    }),
  ],
});
