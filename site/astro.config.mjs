import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://jorgsowa.github.io',
  base: '/mir',
  integrations: [
    starlight({
      customCss: ['./src/styles/custom.css'],
      title: 'mir',
      description: 'A fast, incremental PHP static analyzer written in Rust.',
      logo: {
        light: './src/assets/logo-light.svg',
        dark: './src/assets/logo-dark.svg',
        replacesTitle: false,
      },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/jorgsowa/mir' },
      ],
      editLink: {
        baseUrl: 'https://github.com/jorgsowa/mir/edit/main/site/',
      },
      sidebar: [
        { label: 'Introduction', link: '/' },
        { label: 'Playground', link: '/playground/' },
        {
          label: 'Guides',
          items: [
            { label: 'Getting Started', link: '/guides/getting-started/' },
            { label: 'Configuration', link: '/guides/configuration/' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI', link: '/reference/cli/' },
            {
              label: 'Issues',
              items: [
                { label: 'Overview', link: '/reference/issues/' },
                { label: 'Undefined', autogenerate: { directory: 'reference/issues/undefined' } },
                { label: 'Nullability', autogenerate: { directory: 'reference/issues/nullability' } },
                { label: 'Type Mismatches', autogenerate: { directory: 'reference/issues/type-mismatches' } },
                { label: 'Array', autogenerate: { directory: 'reference/issues/array' } },
                { label: 'Redundancy', autogenerate: { directory: 'reference/issues/redundancy' } },
                { label: 'Dead Code', autogenerate: { directory: 'reference/issues/dead-code' } },
                { label: 'Inheritance', autogenerate: { directory: 'reference/issues/inheritance' } },
                { label: 'Security', autogenerate: { directory: 'reference/issues/security' } },
                { label: 'Generics', autogenerate: { directory: 'reference/issues/generics' } },
                { label: 'Other', autogenerate: { directory: 'reference/issues/other' } },
              ],
            },
            { label: 'Docblock Annotations', link: '/reference/docblock/' },
            { label: 'Architecture', link: '/reference/architecture/' },
          ],
        },
      ],
    }),
  ],
});
