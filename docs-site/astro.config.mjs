// @ts-check
import {defineConfig, envField} from 'astro/config';
import starlight from '@astrojs/starlight';
import svelte from '@astrojs/svelte';

// https://astro.build/config
export default defineConfig({
    site: 'https://octofhir.github.io',
    integrations: [
        svelte(),
        starlight({
            favicon: '/favicon.ico',
            title: 'OctoFHIR FHIRPath',
            description: 'A high-performance FHIRPath implementation in Rust with multiple language bindings.',
            social: [
                {icon: 'github', label: 'GitHub', href: 'https://github.com/octofhir/fhirpath-rs'},
            ],
            sidebar: [
                {
                    label: 'Overview',
                    items: [
                        {label: 'Home', slug: 'index'},
                        {label: 'Introduction', slug: 'introduction'},
                    ],
                },
                {
                    label: 'Getting Started',
                    items: [
                        {label: 'Installation', slug: 'getting-started/installation'},
                        {label: 'Quick Start', slug: 'getting-started/quick-start'},
                    ],
                },
                {
                    label: 'Usage',
                    items: [
                        {label: 'CLI Tool', slug: 'usage/cli'},
                        {label: 'Rust Library', slug: 'usage/rust'},
                        {label: 'Node.js Bindings', slug: 'usage/nodejs'},
                        {label: 'WebAssembly', slug: 'usage/wasm'},
                    ],
                },
                {
                    label: 'Examples',
                    items: [
                        {label: 'Usage Examples', slug: 'examples/usage-examples'},
                    ],
                },
                {
                    label: 'Technical',
                    items: [
                        {label: 'Architecture & Design', slug: 'technical/architecture'},
                        {label: 'Implementation Details', slug: 'technical/implementation'},
                        {label: 'Roadmap & Future Plans', slug: 'technical/roadmap'},
                    ],
                },
                {
                    label: 'Development',
                    items: [
                        {label: 'Contributing', slug: 'development/contributing'},
                        {label: 'Performance', slug: 'development/performance'},
                    ],
                },
                {
                    label: 'Reference',
                    items: [
                        {label: 'Test Compliance', slug: 'reference/test-compliance'},
                    ],
                },
            ],
            components: {
                SiteTitle: "./src/components/SiteTitle.astro",
            }
        }),
    ],
    env: {
        schema: {
            PUBLIC_BASE_URL: envField.string({ context: "client", access: "public" }),
        }
    }
});
