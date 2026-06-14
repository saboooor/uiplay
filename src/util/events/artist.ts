import { UiPlayStoreType } from '../types';

export const regex = /Artist: (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Artist = match[1];

  if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
  UiPlayStore.NowPlaying.Artist = Artist;
};