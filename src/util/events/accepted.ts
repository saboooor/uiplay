import { UiPlayStoreType } from '../types';

export const regex = /Accepted (.*) client on socket (.*)/;
export function execute(match: RegExpMatchArray, UiPlayStore: UiPlayStoreType) {
  const Socket = match[2];
  UiPlayStore.Socket = Socket;
};