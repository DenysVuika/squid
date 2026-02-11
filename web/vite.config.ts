import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import * as fs from 'fs';
import * as path from 'path';
import tailwindcss from '@tailwindcss/vite';

// Plugin to preserve .gitkeep file when emptying outDir
function preserveGitkeep(): Plugin {
  let outDir: string;
  let gitkeepContent: Buffer | null = null;

  return {
    name: 'preserve-gitkeep',
    configResolved(config) {
      outDir = path.resolve(config.root, config.build.outDir);
    },
    buildStart() {
      const gitkeepPath = path.join(outDir, '.gitkeep');
      if (fs.existsSync(gitkeepPath)) {
        gitkeepContent = fs.readFileSync(gitkeepPath);
      }
    },
    closeBundle() {
      if (gitkeepContent !== null) {
        const gitkeepPath = path.join(outDir, '.gitkeep');
        fs.writeFileSync(gitkeepPath, gitkeepContent);
      }
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  base: '/',
  build: {
    outDir: '../static',
    emptyOutDir: true,
  },
  plugins: [react(), preserveGitkeep(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
