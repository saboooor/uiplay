import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import { Event } from '@tauri-apps/api/event';
import type { UiPlayStoreType } from './types';

let DeviceID = '';

export const listenToUxPlayOutput = async (event: Event<string>, UiPlayStore: UiPlayStoreType) => {
  const acceptedRegex = /Accepted (.*) client on socket (.*)/;
  const acceptedRegexMatch = event.payload.match(acceptedRegex);
  if (acceptedRegexMatch) {
    const Socket = acceptedRegexMatch[2];
    UiPlayStore.Socket = Socket;
  }

  const connectionRequestRegex = /connection request from (.*) with deviceID = (.*)/;
  const connectionRequestRegexMatch = event.payload.match(connectionRequestRegex);
  if (connectionRequestRegexMatch) {
    const DeviceName = connectionRequestRegexMatch[1];
    DeviceID = connectionRequestRegexMatch[2];

    if (UiPlayStore.Devices.some(d => d.DeviceID === DeviceID)) {
      // If device already exists, update its name
      const device = UiPlayStore.Devices.find(d => d.DeviceID === DeviceID);
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
        DeviceID,
      });
    }
  }

  const connectionClosedRegex = /Connection closed for socket (.*)/;
  const connectionClosedRegexMatch = event.payload.match(connectionClosedRegex);
  if (connectionClosedRegexMatch) {
    const Socket = connectionClosedRegexMatch[1];

    const device = UiPlayStore.Devices.find(d => d.Socket === Socket);
    if (!device) return;
    device.Socket = undefined;
    device.Connected = false;
  }

  const clientIdentifiedRegex = /Client identified as User-Agent: (.*)/;
  const clientIdentifiedRegexMatch = event.payload.match(clientIdentifiedRegex);
  if (clientIdentifiedRegexMatch) {
    const UserAgent = clientIdentifiedRegexMatch[1];
    const device = UiPlayStore.Devices.find(d => d.DeviceID === DeviceID);

    if (!device) return;
    device.UserAgent = UserAgent;
    device.Connected = true;
  }

  const startAudioConnectionRegex = /start audio connection, format (.*)/;
  const startAudioConnectionRegexMatch = event.payload.match(startAudioConnectionRegex);
  if (startAudioConnectionRegexMatch) {
    const Format = startAudioConnectionRegexMatch[1];

    const device = UiPlayStore.Devices.find(d => d.DeviceID === DeviceID);
    if (!device) return;

    device.Audio = {
      Format,
    };
  }

  const albumRegex = /Album: (.*)/;
  const albumRegexMatch = event.payload.match(albumRegex);
  if (albumRegexMatch) {
    const Album = albumRegexMatch[1];

    if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
    UiPlayStore.NowPlaying.Album = Album;

    try {
      const albumArt = await readFile('uiplay/albumart.png', {
        baseDir: BaseDirectory.Config,
      });
      const base64AlbumArt = btoa(String.fromCharCode(...albumArt));
      console.log('Album art read successfully:', base64AlbumArt);
      if (UiPlayStore.NowPlaying) UiPlayStore.NowPlaying.AlbumArt = `data:image/png;base64,${base64AlbumArt}`;
    }
    catch (error) {
      console.error('Failed to read album art:', error);
    }
  }

  const artistRegex = /Artist: (.*)/;
  const artistRegexMatch = event.payload.match(artistRegex);
  if (artistRegexMatch) {
    const Artist = artistRegexMatch[1];

    if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
    UiPlayStore.NowPlaying.Artist = Artist;
  }

  const genreRegex = /Genre: (.*)/;
  const genreRegexMatch = event.payload.match(genreRegex);
  if (genreRegexMatch) {
    const Genre = genreRegexMatch[1];

    if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
    UiPlayStore.NowPlaying.Genre = Genre;
  }

  const titleRegex = /Title: (.*)/;
  const titleRegexMatch = event.payload.match(titleRegex);
  if (titleRegexMatch) {
    const Title = titleRegexMatch[1];

    if (!UiPlayStore.NowPlaying) UiPlayStore.NowPlaying = {};
    UiPlayStore.NowPlaying.Title = Title;
  }

  const progressRegex = /audio progress (.*)/;
  const progressRegexMatch = event.payload.match(progressRegex);
  if (progressRegexMatch) {
    // min:sec regex
    const regex = /(\d+):(\d+)/g;
    const match = event.payload.matchAll(regex);
    const matches = Array.from(match);
    console.log(matches);

    const progress = matches[0].slice(1).map(Number);
    const remaining = matches[1].slice(1).map(Number);
    const length = matches[2].slice(1).map(Number);

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
  }
};