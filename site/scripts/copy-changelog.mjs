#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, '..', '..', 'CHANGELOG.md');
const dst = join(here, '..', 'src', 'content', 'docs', 'changelog.md');

const body = readFileSync(src, 'utf-8').replace(/^# Changelog\s*/, '');
const frontmatter = [
  '---',
  'title: Changelog',
  'description: Release notes for mir.',
  'tableOfContents:',
  '  maxHeadingLevel: 2',
  '---',
  '',
  '',
].join('\n');

writeFileSync(dst, frontmatter + body);
console.log(`changelog: wrote ${dst}`);
