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
          autogenerate: { directory: '.' },
        },
      ],
    }),
  ],
});
