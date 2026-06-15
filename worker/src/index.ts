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

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};

/**
 * Generates a unique API key for a device using the server's master secret.
 */
async function generateDeviceKey(deviceId: string, secret: string) {
  const encoder = new TextEncoder();
  const data = encoder.encode(`${deviceId}:${secret}`);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // Handle CORS preflight requests
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    const url = new URL(request.url);

    // Endpoint for the client to retrieve their specific API key
    if (url.pathname === '/auth' && request.method === 'POST') {
      // Check for a shared application secret to ensure the request is from the official app
      const appSecret = request.headers.get('X-App-Secret');
      if (!env.APP_SECRET || appSecret !== env.APP_SECRET) {
        return new Response('Forbidden', { status: 403, headers: corsHeaders });
      }

      const deviceId = url.searchParams.get('device_id');
      if (!deviceId) {
        return new Response('Missing device_id', { status: 400, headers: corsHeaders });
      }

      const key = await generateDeviceKey(deviceId, env.AUTH_TOKEN);
      return new Response(key, {
        headers: { ...corsHeaders, 'Content-Type': 'text/plain' },
      });
    }

    // Route matching for the /upload path used in Tauri
    if (url.pathname === '/upload' && request.method === 'POST') {
      // 1. Extract metadata from query string
      const deviceId = url.searchParams.get('device_id');
      if (!deviceId) {
        return new Response('Missing device_id', { status: 400, headers: corsHeaders });
      }

      // 2. Validate Device-Specific Token
      const expectedKey = await generateDeviceKey(deviceId, env.AUTH_TOKEN);
      const auth = request.headers.get('Authorization');
      if (auth !== `Bearer ${expectedKey}`) {
        return new Response('Unauthorized', { status: 401, headers: corsHeaders });
      }

      // 3. Process the file (binary body) and store in R2
      const binaryData = await request.arrayBuffer();
      const storageKey = `artworks/${deviceId}/albumart.png`;

      await env.BUCKET.put(storageKey, binaryData, {
        httpMetadata: { contentType: 'image/png' },
      });

      // 4. Return the public URL for the client to use
      const cdnUrl = `${env.R2_URL}/${storageKey}?t=${Date.now()}`;
      return new Response(cdnUrl, {
        headers: { ...corsHeaders, 'Content-Type': 'text/plain' },
      });
    }

    return new Response('Not Found', { status: 404, headers: corsHeaders });
  },
} satisfies ExportedHandler<Env>;
