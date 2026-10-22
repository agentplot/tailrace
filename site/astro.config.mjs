// The site, built by Astro with Starlight and published to GitHub Pages at
// https://agentplot.github.io/tailrace by .github/workflows/pages.yml.
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';

export default defineConfig({
  site: 'https://agentplot.github.io',
  base: '/tailrace',
  integrations: [
    starlight({
      title: 'tailrace',
      description: 'Build once, sign everything, promote with proof, in your own account.',
      logo: {
        light: './src/assets/mark-light.svg',
        dark: './src/assets/mark-dark.svg',
        alt: 'tailrace',
      },
      favicon: '/favicon.svg',
      // The family's header, status banner and footer from
      // @agentplot/site-theme, which find tailrace by the title above; the
      // start page's hero, with the pipeline drawn under it, where every
      // other page keeps Starlight's.
      components: {
        Header: '@agentplot/site-theme/starlight/Header.astro',
        Banner: '@agentplot/site-theme/starlight/Banner.astro',
        Footer: '@agentplot/site-theme/starlight/Footer.astro',
        Hero: './src/components/Hero.astro',
      },
      // The family's type, layout and accent, then the diagram's palette.
      customCss: ['@agentplot/site-theme/starlight.css', './src/styles/diagrams.css'],
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/agentplot/tailrace' }],
      editLink: { baseUrl: 'https://github.com/agentplot/tailrace/edit/main/site/' },
      plugins: [starlightLinksValidator()],
      sidebar: [
        { label: 'Start', link: '/' },
        { label: 'How it works', link: '/how-it-works/' },
        { label: 'How it fits', link: '/how-it-fits/' },
        {
          label: 'Guides',
          items: [
            { label: 'Self-hosting', link: '/guides/self-hosting/' },
            { label: 'Integrations', items: [{ autogenerate: { directory: 'guides/integrations' } }] },
          ],
        },
        { label: 'Design', link: '/design/' },
        { label: 'Roadmap', link: '/roadmap/' },
      ],
    }),
  ],
});
