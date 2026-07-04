#!/usr/bin/env node
import { readdir, readFile, stat } from 'node:fs/promises';
import { dirname, extname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(scriptsDir, '..');
const distRoot = resolve(siteRoot, process.argv[2] ?? 'dist');

const linkAttrPattern = /\b(?:href|src)\s*=\s*(["'])(.*?)\1/gis;
const srcsetAttrPattern = /\bsrcset\s*=\s*(["'])(.*?)\1/gis;
const idAttrPattern = /\bid\s*=\s*(["'])(.*?)\1/gis;

function decodeHtml(value) {
  return value
    .replace(/&amp;/g, '&')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&apos;/g, "'")
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>');
}

function decodePathPart(value) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function isSkippableUrl(rawUrl) {
  const url = rawUrl.trim();
  if (!url) return true;
  if (url.startsWith('//')) return true;
  return /^[a-z][a-z0-9+.-]*:/i.test(url);
}

function isInsideDist(path) {
  const rel = relative(distRoot, path);
  return rel === '' || (!rel.startsWith('..') && !rel.startsWith('/'));
}

async function existsAsFile(path) {
  try {
    return (await stat(path)).isFile();
  } catch {
    return false;
  }
}

async function existsAsDirectory(path) {
  try {
    return (await stat(path)).isDirectory();
  } catch {
    return false;
  }
}

async function* walkHtml(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkHtml(path);
    } else if (entry.isFile() && entry.name.endsWith('.html')) {
      yield path;
    }
  }
}

function extractLinks(html) {
  const links = [];
  for (const match of html.matchAll(linkAttrPattern)) {
    links.push(decodeHtml(match[2]));
  }

  for (const match of html.matchAll(srcsetAttrPattern)) {
    const entries = decodeHtml(match[2]).split(',');
    for (const entry of entries) {
      const [url] = entry.trim().split(/\s+/);
      if (url) links.push(url);
    }
  }

  return links;
}

function extractIds(html) {
  const ids = new Set();
  for (const match of html.matchAll(idAttrPattern)) {
    ids.add(decodeHtml(match[2]));
  }
  return ids;
}

async function resolveLocalTarget(fromFile, link) {
  const hashIndex = link.indexOf('#');
  const beforeHash = hashIndex >= 0 ? link.slice(0, hashIndex) : link;
  const fragment = hashIndex >= 0 ? link.slice(hashIndex + 1).split('?')[0] : '';
  const pathPart = beforeHash.split('?')[0];

  let candidate;
  if (!pathPart) {
    candidate = fromFile;
  } else if (pathPart.startsWith('/')) {
    candidate = resolve(distRoot, decodePathPart(pathPart.replace(/^\/+/, '')));
  } else {
    candidate = resolve(dirname(fromFile), decodePathPart(pathPart));
  }

  if (!isInsideDist(candidate)) {
    return { ok: false, reason: 'points outside site/dist' };
  }

  let targetFile = null;
  if (await existsAsFile(candidate)) {
    targetFile = candidate;
  } else if (await existsAsDirectory(candidate) && await existsAsFile(join(candidate, 'index.html'))) {
    targetFile = join(candidate, 'index.html');
  } else if (!extname(candidate) && await existsAsFile(`${candidate}.html`)) {
    targetFile = `${candidate}.html`;
  }

  if (!targetFile) {
    return { ok: false, reason: `missing target ${relative(distRoot, candidate) || '.'}` };
  }

  if (fragment && targetFile.endsWith('.html')) {
    const targetHtml = await readFile(targetFile, 'utf8');
    const ids = extractIds(targetHtml);
    const decodedFragment = decodePathPart(fragment);
    if (!ids.has(fragment) && !ids.has(decodedFragment)) {
      return {
        ok: false,
        reason: `missing fragment #${fragment} in ${relative(distRoot, targetFile)}`,
      };
    }
  }

  return { ok: true };
}

async function main() {
  if (!(await existsAsDirectory(distRoot))) {
    console.error(`Docs output directory does not exist: ${distRoot}`);
    console.error('Run `bun run build` before `bun run check:links`.');
    process.exit(1);
  }

  const htmlFiles = [];
  for await (const file of walkHtml(distRoot)) htmlFiles.push(file);

  if (htmlFiles.length === 0) {
    console.error(`No HTML files found under ${distRoot}`);
    process.exit(1);
  }

  const failures = [];
  let checked = 0;

  for (const file of htmlFiles) {
    const html = await readFile(file, 'utf8');
    for (const link of extractLinks(html)) {
      if (isSkippableUrl(link)) continue;
      checked += 1;
      const result = await resolveLocalTarget(file, link);
      if (!result.ok) {
        failures.push({ file, link, reason: result.reason });
      }
    }
  }

  if (failures.length > 0) {
    console.error(`Found ${failures.length} broken internal docs link(s):`);
    for (const failure of failures) {
      console.error(
        `- ${relative(siteRoot, failure.file)} -> ${failure.link}: ${failure.reason}`,
      );
    }
    process.exit(1);
  }

  console.log(
    `Checked ${checked} internal docs link(s) across ${htmlFiles.length} HTML file(s) in ${relative(siteRoot, distRoot)}.`,
  );
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
