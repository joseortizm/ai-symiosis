import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import typescript from '@typescript-eslint/eslint-plugin';
import parser from '@typescript-eslint/parser';
import svelteParser from 'svelte-eslint-parser';
import globals from 'globals';

export default [
  js.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.js', '**/*.ts', '**/*.svelte'],
    languageOptions: {
      parser: parser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: 'module',
        extraFileExtensions: ['.svelte']
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        $state: 'readonly',
        $effect: 'readonly',
        $derived: 'readonly',
        $props: 'readonly',
        $bindable: 'readonly',
        $inspect: 'readonly'
      }
    },
    plugins: {
      '@typescript-eslint': typescript
    },
    rules: {
      ...typescript.configs.recommended.rules,
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      'svelte/no-unused-svelte-ignore': 'error',
      'svelte/no-dom-manipulating': 'error',
      'svelte/no-at-html-tags': 'error',
      'svelte/prefer-svelte-reactivity': 'error',
      'no-console': ['warn', { allow: ['warn', 'error'] }]
    }
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: parser
      }
    }
  },
  {
    files: ['**/tests/**/*.ts', '**/*.test.ts', '**/*.spec.ts'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.vitest,
        describe: 'readonly',
        it: 'readonly',
        test: 'readonly',
        expect: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
        vi: 'readonly'
      }
    }
  },
  {
    files: ['**/Editor.svelte'],
    rules: {
      'svelte/no-dom-manipulating': 'off'
    }
  },
  {
    files: ['**/NoteView.svelte'],
    rules: {
      'svelte/no-at-html-tags': 'off'
    }
  },
  {
    files: ['**/contentHighlighting.svelte.ts'],
    rules: {
      'svelte/prefer-svelte-reactivity': 'off'
    }
  },
  {
    ignores: [
      'build/',
      '.svelte-kit/',
      'dist/',
      'docs/',
      'node_modules/',
      'src-tauri/target/',
      '.tauri/',
      'coverage/',
      'package-lock.json',
      'pnpm-lock.yaml',
      'yarn.lock'
    ]
  }
];
