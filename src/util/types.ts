
export type UiPlayStoreType = {
  Settings: {
    Name: string;
  };
  Socket?: string;
  NowPlaying?: {
    AlbumArt?: string;
    Album?: string;
    Artist?: string;
    Title?: string;
    Genre?: string;
    Progress?: {
      min: number;
      sec: number;
    };
    Remaining?: {
      min: number;
      sec: number;
    };
    Length?: {
      min: number;
      sec: number;
    };
  };
  Devices: {
    Socket?: string;
    DeviceID: string;
    DeviceName: string;
    Connected?: boolean;
    UserAgent?: string;
    Audio?: {
      Format: string;
    }
  }[];
  TerminalOpen?: boolean;
};