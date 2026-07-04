import { defineRouteMiddleware } from '@astrojs/starlight/route-data';

const canonicalDocsEditBase = 'https://github.com/Algorant/tandem/edit/main/docs/';
const generatedDocsPrefix = 'src/content/docs/';
const generatedDocsMarker = '/src/content/docs/';

function getCanonicalDocsPath(filePath: string) {
  const normalizedPath = filePath.replaceAll('\\', '/');

  if (normalizedPath.startsWith(generatedDocsPrefix)) {
    return normalizedPath.slice(generatedDocsPrefix.length);
  }

  const markerIndex = normalizedPath.lastIndexOf(generatedDocsMarker);
  if (markerIndex !== -1) {
    return normalizedPath.slice(markerIndex + generatedDocsMarker.length);
  }

  return undefined;
}

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;

  if (route.entry.data.editUrl === false) {
    return;
  }

  const canonicalPath = getCanonicalDocsPath(route.entry.filePath);
  if (!canonicalPath) {
    return;
  }

  route.editUrl = new URL(canonicalPath, canonicalDocsEditBase);
});
