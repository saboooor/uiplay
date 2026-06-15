import { cache } from '../listen';
import { UiPlayStoreType } from '../types';

export const regex = /Client identified as User-Agent: (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const UserAgent = match[1];

  const device = UiPlayStore.Devices.find(d => d.DeviceID === cache.DeviceID);
  if (!device) return;
  device.UserAgent = UserAgent;
  device.Connected = true;
};