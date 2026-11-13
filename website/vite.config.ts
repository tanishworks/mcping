import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Base path must match the GitHub Pages URL: https://tanishworks.github.io/mcping/
export default defineConfig({
    base: '/mcping/',
    plugins: [react()],
});
