import { cache } from '../listen';
import { UiPlayStoreType } from '../types';

export const regex = /start audio connection, format (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Format = match[1];

  const device = UiPlayStore.Devices.find(d => d.DeviceID === cache.DeviceID);
  if (!device) return;

  device.Audio = {
    Format,
  };
};