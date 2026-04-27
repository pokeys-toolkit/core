// @ts-check
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';

export default defineConfig({
  site: 'https://pokeys-toolkit.github.io',
  base: '/core',
  integrations: [react()],
});
