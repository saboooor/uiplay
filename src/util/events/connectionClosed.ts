import { UiPlayStoreType } from '../types';

export const regex = /Connection closed for socket (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Socket = match[1];

  const device = UiPlayStore.Devices.find(d => d.Socket === Socket);
  if (!device) return;
  device.Socket = undefined;
  device.Connected = false;
};