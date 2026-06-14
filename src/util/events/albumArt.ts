import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import { cache } from '../listen';
import { UiPlayStoreType } from '../types';
import { invoke } from '@tauri-apps/api/core';

export const regex = /coverart size (.*)/;
export async function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  try {
    const albumArt = await readFile('uiplay/albumart.png', {
      baseDir: BaseDirectory.Config,
    });

    // Deduplicate: Only upload if the art is different from the last one we processed
    const isNewArt = !cache.lastArt ||
      cache.lastArt.length !== albumArt.length ||
      albumArt.some((byte, i) => byte !== cache.lastArt![i]);

    if (!isNewArt) return;
    cache.lastArt = albumArt;

    try {
      // Call the Rust command to handle the network request securely
      const cdnUrl = await invoke<string>('upload_to_cdn', { deviceId: cache.DeviceID });

      // Append a timestamp to the URL to force Discord to bypass its cache if the ID remains the same
      const freshUrl = `${cdnUrl}?t=${Date.now()}`;
      const base64AlbumArt = btoa(String.fromCharCode(...albumArt));

      if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
      UiPlayStore.NowPlaying.AlbumArt = `data:image/png;base64,${base64AlbumArt}`;
      console.log('Uploaded to CDN:', freshUrl);
    } catch (uploadError) {
      console.error('Failed to upload to CDN, falling back to local base64:', uploadError);
    }
  }
  catch (error) {
    console.error('Failed to read album art:', error);
  }
};