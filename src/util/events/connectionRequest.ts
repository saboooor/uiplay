import { cache } from '../listen';
import { UiPlayStoreType } from '../types';

export const regex = /connection request from (.*) with deviceID = (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const DeviceName = match[1];
  cache.DeviceID = match[2];

  if (UiPlayStore.Devices.some(d => d.DeviceID === cache.DeviceID)) {
    // If device already exists, update its name
    const device = UiPlayStore.Devices.find(d => d.DeviceID === cache.DeviceID);
    if (device) {
      device.Socket = UiPlayStore.Socket;
      device.DeviceName = DeviceName;
    }
  }
  else {
    // If device does not exist, add it
    UiPlayStore.Devices.push({
      Socket: UiPlayStore.Socket,
      DeviceName,
      DeviceID: cache.DeviceID,
    });
  }
};