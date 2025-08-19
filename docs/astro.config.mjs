// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
	site: 'https://pokeys-toolkit.github.io',
	base: '/core',
	integrations: [
		tailwind({ applyBaseStyles: false }),
		starlight({
			title: 'PoKeys Core Library',
			description: 'Pure Rust implementation of the PoKeysLib for controlling PoKeys devices',
			customCss: ['./src/styles/custom.css'],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/pokeys-toolkit/core' }],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'guides/introduction' },
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'Quick Start', slug: 'guides/quick-start' },
					],
				},
				{
					label: 'API Reference',
					autogenerate: { directory: 'reference' },
				},
				{
					label: 'Examples',
					autogenerate: { directory: 'examples' },
				},
			],
		}),
	],
});
