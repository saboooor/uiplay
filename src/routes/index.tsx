import { $, component$, useContext, useSignal } from '@qwik.dev/core';
import { type DocumentHead } from '@qwik.dev/router';
import { UiPlayStoreContext } from './layout';
import Airplay from 'lucide-icons-qwik/icons/Airplay';
import Laptop from 'lucide-icons-qwik/icons/Laptop';
import Smartphone from 'lucide-icons-qwik/icons/Smartphone';
import Trash from 'lucide-icons-qwik/icons/Trash';
import SkipBack from 'lucide-icons-qwik/icons/SkipBack';
import Play from 'lucide-icons-qwik/icons/Play';
import SkipForward from 'lucide-icons-qwik/icons/SkipForward';
import { invoke } from '@tauri-apps/api/core';

export default component$(() => {
  const UiPlayStore = useContext(UiPlayStoreContext);
  const controlError = useSignal('');

  const controlPlayback = $(
    async (control: 'previous' | 'playpause' | 'next') => {
      controlError.value = '';
      try {
        await invoke('control_shairport', { control });
      } catch (error) {
        controlError.value = String(error);
      }
    }
  );

  return (
    <>
      {!UiPlayStore.NowPlaying && (
        <div
          class={{
            'flex flex-col items-center gap-2 text-center': true,
          }}
        >
          <Airplay size={100} strokeWidth={2} />
          <h1
            class={{
              'from-luminescent-500 to-luminescent-100 mt-4 bg-linear-to-br bg-clip-text! text-5xl font-bold tracking-tight text-transparent': true,
              'motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-600': true,
            }}
          >
            UiPlay
          </h1>
          <p class="text-lum-text-secondary motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-800 mt-2 text-lg">
            A UxPlay wrapper that forwards music data to mpris and Discord RPC.
          </p>
          <p class="motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-1000 text-gray-500">
            Go ahead and airplay! Info will be displayed here.
            <br />
            You can also open the terminal to see logs and debug info.
          </p>
        </div>
      )}

      <div class="motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-1000 mt-8 flex flex-row items-center gap-16 lg:gap-[10vw]">
        {UiPlayStore.NowPlaying?.AlbumArt && (
          <div class="flex">
            <img
              src={UiPlayStore.NowPlaying.AlbumArt}
              alt="Album Art"
              width={128}
              height={128}
              class="rounded-lum-4 flex aspect-square h-[50vh] w-[50vh] object-cover"
            />
          </div>
        )}
        <div class="flex flex-1 flex-col lg:gap-2">
          {UiPlayStore.NowPlaying && (
            <>
              <h2 class="flex items-center gap-2 text-2xl font-bold tracking-tight lg:text-3xl">
                {UiPlayStore.NowPlaying.Title}
              </h2>
              {UiPlayStore.NowPlaying.Artist && (
                <p class="text-2xl font-semibold tracking-tight text-gray-400 lg:text-3xl">
                  {UiPlayStore.NowPlaying.Artist}
                </p>
              )}
              <p class="text-xl font-light tracking-tight text-gray-500 lg:text-2xl">
                {UiPlayStore.NowPlaying.Album && (
                  <span>{UiPlayStore.NowPlaying.Album}</span>
                )}
                {' - '}
                {UiPlayStore.NowPlaying.Genre && (
                  <span>{UiPlayStore.NowPlaying.Genre}</span>
                )}
              </p>
              {UiPlayStore.NowPlaying.Progress &&
                UiPlayStore.NowPlaying.Remaining &&
                UiPlayStore.NowPlaying.Length && (
                  <div class="lum-bg-gray-700/20 mt-6 h-2 w-full overflow-hidden rounded-full">
                    <div
                      class="h-full border-none bg-gray-200/20 transition-all duration-1000 ease-linear"
                      style={{
                        width: `${
                          ((UiPlayStore.NowPlaying.Progress.min * 60 +
                            UiPlayStore.NowPlaying.Progress.sec) /
                            (UiPlayStore.NowPlaying.Length.min * 60 +
                              UiPlayStore.NowPlaying.Length.sec)) *
                          100
                        }%`,
                      }}
                    />
                  </div>
                )}
              {UiPlayStore.NowPlaying.Progress &&
                UiPlayStore.NowPlaying.Length && (
                  <div class="mt-1 flex w-full justify-between text-gray-500">
                    <p>
                      {UiPlayStore.NowPlaying.Progress.min}:
                      {UiPlayStore.NowPlaying.Progress.sec
                        .toString()
                        .padStart(2, '0')}
                    </p>
                    <p>
                      {UiPlayStore.NowPlaying.Length.min}:
                      {UiPlayStore.NowPlaying.Length.sec
                        .toString()
                        .padStart(2, '0')}
                    </p>
                  </div>
                )}
              <div class="mt-5 flex flex-wrap items-center justify-center gap-3">
                <button
                  type="button"
                  aria-label="Previous track"
                  title="Previous track"
                  class="lum-btn lum-bg-transparent rounded-full p-3"
                  onClick$={() => controlPlayback('previous')}
                >
                  <SkipBack size={24} />
                </button>
                <button
                  type="button"
                  aria-label="Play or pause"
                  title="Play or pause"
                  class="lum-btn lum-bg-gray-700/30 rounded-full p-4"
                  onClick$={() => controlPlayback('playpause')}
                >
                  <Play size={24} class="fill-current" />
                </button>
                <button
                  type="button"
                  aria-label="Next track"
                  title="Next track"
                  class="lum-btn lum-bg-transparent rounded-full p-3"
                  onClick$={() => controlPlayback('next')}
                >
                  <SkipForward size={24} />
                </button>
                {controlError.value && (
                  <p
                    class="basis-full text-center text-sm text-red-300"
                    role="alert"
                  >
                    {controlError.value}
                  </p>
                )}
              </div>
            </>
          )}
          <div class="mt-4 flex">
            {UiPlayStore.Devices.map((device) => (
              <div
                key={device.DeviceID}
                class="mt-4 flex flex-row items-center gap-2"
              >
                <div
                  class={{
                    'ml-2 inline-block h-2 w-2 rounded-full': true,
                    'bg-orange-300/20': !device.Connected,
                    'bg-green-300/50': device.Connected,
                  }}
                />
                {device.DeviceName.includes('Phone') && (
                  <Smartphone size={30} class="text-lum-text" />
                )}
                {device.DeviceName.includes('Mac') && (
                  <Laptop size={30} class="text-lum-text" />
                )}
                <div class="flex-1">
                  <p class="text-lg font-semibold tracking-tight">
                    {device.DeviceName}
                  </p>
                  {device.Audio && device.Audio.Format && (
                    <p class="text-sm text-orange-200">{device.Audio.Format}</p>
                  )}
                  <p class="text-xs text-gray-400">
                    {device.DeviceID}
                    {device.UserAgent && (
                      <span class="ml-1 text-xs text-gray-400">
                        {device.UserAgent}
                      </span>
                    )}
                  </p>
                </div>
                {!device.Connected && (
                  <button
                    class="lum-btn lum-bg-transparent p-2 text-red-300 hover:text-red-400"
                    onClick$={() => {
                      UiPlayStore.Devices = UiPlayStore.Devices.filter(
                        (d) => d.DeviceID !== device.DeviceID
                      );
                    }}
                  >
                    <Trash size={20} />
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  );
});

export const head: DocumentHead = {
  title: 'UiPlay',
  meta: [
    {
      name: 'description',
      content: 'A UxPlay wrapper.',
    },
  ],
};
