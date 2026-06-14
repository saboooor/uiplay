import { UiPlayStoreType } from '../types';

export const regex = /Album: (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Album = match[1];

  if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
  UiPlayStore.NowPlaying.Album = Album;
};