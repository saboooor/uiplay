import { Event } from '@tauri-apps/api/event';
import type { UiPlayStoreType } from './types';
import regexes from './events/index';

export const cache = {
  DeviceID: '',
  lastArt: null as Uint8Array | null,
};

export const listenToUxPlayOutput = async (event: Event<string>, UiPlayStore: UiPlayStoreType) => {
  const regex = regexes.find(r => r.regex.test(event.payload));
  if (!regex) return;

  regex.execute(
    event.payload.match(regex.regex)!,
    UiPlayStore,
  );
};