import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load the FHIRPath grammar
const fhirpathGrammar = JSON.parse(readFileSync(join(__dirname, 'src/content/grammars/fhirpath.tmGrammar.json'), 'utf8'));

// Create the language object compatible with Shiki v3
const fhirpathLanguage = {
  id: 'fhirpath',
  scopeName: 'source.fhirpath',
  aliases: ['fhirpath', 'fhir'],
  ...fhirpathGrammar
};

// Astro 5 + Starlight configuration
export default defineConfig({
  site: 'https://octofhir.github.io',
  base: '/fhirpath-rs',
  integrations: [
    starlight({
      title: 'FHIRPath-rs',
      description: 'Fast, spec-compliant FHIRPath engine in Rust',
      logo: {
        src: './public/logo.png',
        replacesTitle: false,
      },
      favicon: '/favicon.ico',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/octofhir/fhirpath-rs' },
      ],
      expressiveCode: {
        shiki: {
          langs: [fhirpathLanguage],
        },
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'index' },
          ],
        },
        {
          label: 'Function Library',
          autogenerate: { directory: 'functions' },
        },
        {
          label: 'Error Codes',
          autogenerate: { directory: 'errors' },
        },
      ],
      editLink: {
        baseUrl: 'https://github.com/octofhir/fhirpath-rs/edit/main/',
      },
      defaultLocale: 'en',
    }),
  ],
});
