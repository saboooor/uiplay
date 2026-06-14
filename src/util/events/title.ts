import { UiPlayStoreType } from '../types';

export const regex = /Title: (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Title = match[1];

  if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
  UiPlayStore.NowPlaying.Title = Title;
};