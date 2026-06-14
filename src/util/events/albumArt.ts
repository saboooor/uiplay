import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import { cache } from '../listen';
import { UiPlayStoreType } from '../types';

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

    const base64AlbumArt = btoa(String.fromCharCode(...albumArt));

    if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
    UiPlayStore.NowPlaying.AlbumArt = `data:image/png;base64,${base64AlbumArt}`;
  }
  catch (error) {
    console.error('Failed to read album art:', error);
  }
};