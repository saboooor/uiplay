import { UiPlayStoreType } from '../types';

const MinSecRegex = /(\d+):(\d+)/g;

export const regex = /audio progress (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const MinSecMatch = match[0].matchAll(MinSecRegex);
  const MinSecMatches = Array.from(MinSecMatch);

  const progress = MinSecMatches[0].slice(1).map(Number);
  const remaining = MinSecMatches[1].slice(1).map(Number);
  const length = MinSecMatches[2].slice(1).map(Number);

  if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
  UiPlayStore.NowPlaying.Progress = {
    min: progress[0],
    sec: progress[1],
  };
  UiPlayStore.NowPlaying.Remaining = {
    min: remaining[0],
    sec: remaining[1],
  };
  UiPlayStore.NowPlaying.Length = {
    min: length[0],
    sec: length[1],
  };
};