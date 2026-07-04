import { cp, mkdir, readdir, rm } from 'node:fs/promises';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(siteRoot, '..', '..');
const sourceDir = join(repoRoot, 'docs');
const targetDir = join(repoRoot, 'site', 'src', 'content', 'docs');
const astroCacheDir = join(repoRoot, 'site', '.astro');
const keepFiles = new Set(['.gitignore', 'README.txt']);

async function* walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walk(path);
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      yield path;
    }
  }
}

// Starlight renders code fences during content collection. Clear Astro's
// generated content cache before syncing so theme/Expressive Code changes do
// not leave stale rendered HTML pointing at removed hashed stylesheets.
await rm(astroCacheDir, { recursive: true, force: true });

await mkdir(targetDir, { recursive: true });

for (const entry of await readdir(targetDir, { withFileTypes: true })) {
  if (!keepFiles.has(entry.name)) {
    await rm(join(targetDir, entry.name), { recursive: true, force: true });
  }
}

let copied = 0;
for await (const source of walk(sourceDir)) {
  const destination = join(targetDir, relative(sourceDir, source));
  await mkdir(dirname(destination), { recursive: true });
  await cp(source, destination);
  copied += 1;
}

console.log(`Synced ${copied} Markdown file(s) from ${relative(repoRoot, sourceDir)} to ${relative(repoRoot, targetDir)}.`);
