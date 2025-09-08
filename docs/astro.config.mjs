import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// Astro 5 + Starlight configuration
export default defineConfig({
  site: 'https://octofhir.github.io',
  base: '/fhirpath-rs',
  integrations: [
    starlight({
      title: 'FHIRPath-rs',
      description: 'Fast, spec-compliant FHIRPath engine in Rust',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/octofhir/fhirpath-rs' },
      ],
      sidebar: [
        {
          label: 'Docs',
          items: [
            { label: 'Introduction', link: '/' },
            { label: 'Function Library', link: '/functions/' },
            { label: 'Error Codes', link: '/errors/' },
          ],
        },
      ],
      editLink: {
        baseUrl: 'https://github.com/octofhir/fhirpath-rs/edit/main/',
      },
    }),
  ],
});
