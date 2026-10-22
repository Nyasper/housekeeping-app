import { readFileSync } from 'node:fs';

import { plugin } from 'bun';
import { compileModule } from 'svelte/compiler';
import ts from 'typescript';

/**
 * Bun's test runner does not know about Svelte runes. This loader compiles
 * `*.svelte.ts` modules with `compileModule` so unit tests can import services
 * that use `$state` and `$derived`.
 */
plugin({
  name: 'svelte-module-loader',
  setup(builder) {
    builder.onLoad({ filter: /\.svelte\.(ts|js)$/ }, ({ path }) => {
      // Never transform dependencies such as `svelte/internal`.
      if (path.includes('node_modules')) {
        return;
      }

      const source = readFileSync(path, 'utf8');
      const javascript = path.endsWith('.ts')
        ? ts.transpileModule(source, {
            compilerOptions: {
              module: ts.ModuleKind.ESNext,
              target: ts.ScriptTarget.ES2022,
              verbatimModuleSyntax: true
            },
            fileName: path
          }).outputText
        : source;

      const compiled = compileModule(javascript, { filename: path, dev: false });

      return {
        contents: compiled.js.code,
        loader: 'js'
      };
    });
  }
});
