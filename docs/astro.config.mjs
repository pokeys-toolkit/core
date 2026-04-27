// @ts-check
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';
import react from '@astrojs/react';

export default defineConfig({
  site: 'https://pokeys-toolkit.github.io',
  base: '/core',
  integrations: [react()],
  vite: {
    plugins: [tailwindcss()],
  },
});
