import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

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
