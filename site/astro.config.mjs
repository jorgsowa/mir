import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';

const SITE = 'https://jorgsowa.github.io';
const BASE = '/mir';
const OG_IMAGE = `${SITE}${BASE}/og-cover.png`;

export default defineConfig({
  site: SITE,
  base: BASE,
  integrations: [
    sitemap(),
    starlight({
      customCss: ['./src/styles/custom.css'],
      title: 'mir',
      description: 'A fast, incremental PHP static analyzer written in Rust.',
      logo: {
        src: './src/assets/logo-hero.png',
        replacesTitle: true,
      },
      head: [
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.googleapis.com' } },
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' } },
        { tag: 'meta', attrs: { property: 'og:type', content: 'website' } },
        { tag: 'meta', attrs: { property: 'og:site_name', content: 'mir' } },
        { tag: 'meta', attrs: { property: 'og:image', content: OG_IMAGE } },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'meta', attrs: { name: 'twitter:image', content: OG_IMAGE } },
        { tag: 'meta', attrs: { name: 'theme-color', content: '#080c14' } },
      ],
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
                { label: 'Undefined',       items: [{ autogenerate: { directory: 'reference/issues/undefined' } }] },
                { label: 'Nullability',     items: [{ autogenerate: { directory: 'reference/issues/nullability' } }] },
                { label: 'Type Mismatches', items: [{ autogenerate: { directory: 'reference/issues/type-mismatches' } }] },
                { label: 'Array',           items: [{ autogenerate: { directory: 'reference/issues/array' } }] },
                { label: 'Redundancy',      items: [{ autogenerate: { directory: 'reference/issues/redundancy' } }] },
                { label: 'Dead Code',       items: [{ autogenerate: { directory: 'reference/issues/dead-code' } }] },
                { label: 'Inheritance',     items: [{ autogenerate: { directory: 'reference/issues/inheritance' } }] },
                { label: 'Security',        items: [{ autogenerate: { directory: 'reference/issues/security' } }] },
                { label: 'Generics',        items: [{ autogenerate: { directory: 'reference/issues/generics' } }] },
                { label: 'Other',           items: [{ autogenerate: { directory: 'reference/issues/other' } }] },
              ],
            },
            { label: 'Docblock Annotations', link: '/reference/docblock/' },
            { label: 'Architecture', link: '/reference/architecture/' },
          ],
        },
        { label: 'Changelog', link: '/changelog/' },
      ],
    }),
  ],
});
