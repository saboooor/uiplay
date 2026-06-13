/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.jsonc`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Route matching for the /upload path used in Tauri
    if (url.pathname === '/upload' && request.method === 'POST') {
      // 1. Validate Secret Token
      const auth = request.headers.get('Authorization');
      if (auth !== `Bearer ${env.AUTH_TOKEN}`) {
        return new Response('Unauthorized', { status: 401 });
      }

      // 2. Extract metadata from query string
      const deviceId = url.searchParams.get('device_id');
      if (!deviceId) {
        return new Response('Missing device_id', { status: 400 });
      }

      // 3. Process the file (binary body) and store in R2
      const binaryData = await request.arrayBuffer();
      const storageKey = `artworks/${deviceId}/albumart.png`;

      await env.BUCKET.put(storageKey, binaryData, {
        httpMetadata: { contentType: 'image/png' },
      });

      // 4. Return the public URL for the client to use
      const cdnUrl = `${env.R2_URL}/${storageKey}`;
      return new Response(cdnUrl);
    }

    return new Response('Not Found', { status: 404 });
  },
} satisfies ExportedHandler<Env>;
