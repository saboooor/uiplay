import { UiPlayStoreType } from '../types';

export const regex = /Genre: (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Genre = match[1];

  if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
  UiPlayStore.NowPlaying.Genre = Genre;
};