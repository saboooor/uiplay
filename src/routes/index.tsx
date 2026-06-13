import { component$, useContext } from '@qwik.dev/core';
import { type DocumentHead } from '@qwik.dev/router';
import { UiPlayStoreContext } from './layout';
import Airplay from 'lucide-icons-qwik/icons/Airplay';
import Laptop from 'lucide-icons-qwik/icons/Laptop';
import Smartphone from 'lucide-icons-qwik/icons/Smartphone';
import Trash from 'lucide-icons-qwik/icons/Trash';

export default component$(() => {
  const UiPlayStore = useContext(UiPlayStoreContext);

  return (
    <>
      {!UiPlayStore.NowPlaying &&
        <div class={{
          'flex flex-col items-center text-center gap-2': true,
        }}>
          <Airplay size={100} strokeWidth={2} />
          <h1 class={{
            'text-5xl font-bold tracking-tight mt-4 text-transparent bg-clip-text! bg-linear-to-br from-luminescent-500 to-luminescent-100': true,
            'motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-600': true,
          }}>
            UiPlay
          </h1>
          <p class="text-lg text-lum-text-secondary mt-2 motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-800">
            A UxPlay wrapper that forwards music data to mpris and Discord RPC.
          </p>
          <p class="text-gray-500 motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-1000">
            Go ahead and airplay! Info will be displayed here.<br/>
            You can also open the terminal to see logs and debug info.
          </p>
        </div>
      }

      <div class="flex flex-row items-center mt-8 gap-16 lg:gap-[10vw] motion-safe:slide-in-from-top-16 animate-in fade-in motion-safe:anim-duration-1000">
        {UiPlayStore.NowPlaying?.AlbumArt &&
          <div class="flex">
            <img
              src={UiPlayStore.NowPlaying.AlbumArt}
              alt="Album Art"
              width={128}
              height={128}
              class="w-[50vh] h-[50vh] flex aspect-square rounded-lum-4 object-cover"
            />
          </div>
        }
        <div class="flex flex-col flex-1 lg:gap-2">
          {UiPlayStore.NowPlaying && <>
            <h2 class="text-2xl lg:text-3xl font-bold tracking-tight flex items-center gap-2">
              {UiPlayStore.NowPlaying.Title}
            </h2>
            {UiPlayStore.NowPlaying.Artist &&
              <p class="text-2xl lg:text-3xl text-gray-400 font-semibold tracking-tight">
                {UiPlayStore.NowPlaying.Artist}
              </p>
            }
            <p class="text-xl lg:text-2xl text-gray-500 font-light tracking-tight">
              {UiPlayStore.NowPlaying.Album &&
                <span>
                  {UiPlayStore.NowPlaying.Album}
                </span>
              }
              {' - '}
              {UiPlayStore.NowPlaying.Genre &&
                <span>
                  {UiPlayStore.NowPlaying.Genre}
                </span>
              }
            </p>
            {UiPlayStore.NowPlaying.Progress && UiPlayStore.NowPlaying.Remaining && UiPlayStore.NowPlaying.Length &&
              <div class="w-full h-2 lum-bg-gray-700/20 rounded-full mt-6 overflow-hidden">
                <div class="transition-all duration-1000 ease-linear h-full bg-gray-200/20 border-none" style={{
                  width: `${
                    (UiPlayStore.NowPlaying.Progress.min * 60 + UiPlayStore.NowPlaying.Progress.sec)
                    /
                    (UiPlayStore.NowPlaying.Length.min * 60 + UiPlayStore.NowPlaying.Length.sec) * 100
                  }%`,
                }} />
              </div>
            }
            {UiPlayStore.NowPlaying.Progress && UiPlayStore.NowPlaying.Length &&
              <div class="flex w-full justify-between text-gray-500 mt-1">
                <p>
                  {UiPlayStore.NowPlaying.Progress.min}:{UiPlayStore.NowPlaying.Progress.sec.toString().padStart(2, '0')}
                </p>
                <p>
                  {UiPlayStore.NowPlaying.Length.min}:{UiPlayStore.NowPlaying.Length.sec.toString().padStart(2, '0')}
                </p>
              </div>
            }
          </>}
          <div class="flex mt-4">
            {UiPlayStore.Devices.map((device) => (
              <div key={device.DeviceID} class="flex flex-row items-center gap-2 mt-4">
                <div class={{
                  'inline-block w-2 h-2 rounded-full ml-2': true,
                  'bg-orange-300/20': !device.Connected,
                  'bg-green-300/50': device.Connected,
                }}/>
                {device.DeviceName.includes('Phone') &&
                  <Smartphone size={30} class="text-lum-text " />
                }
                {device.DeviceName.includes('Mac') &&
                  <Laptop size={30} class="text-lum-text " />
                }
                <div class="flex-1">
                  <p class="font-semibold text-lg tracking-tight">
                    {device.DeviceName}
                  </p>
                  {device.Audio && device.Audio.Format &&
                    <p class="text-orange-200 text-sm">
                      {device.Audio.Format}
                    </p>
                  }
                  <p class="text-gray-400 text-xs">
                    {device.DeviceID}
                    { device.UserAgent &&
                      <span class="text-gray-400 text-xs ml-1">
                        {device.UserAgent}
                      </span>
                    }
                  </p>
                </div>
                {!device.Connected &&
                  <button class="lum-btn lum-bg-transparent p-2 text-red-300 hover:text-red-400"
                    onClick$={() => {
                      UiPlayStore.Devices = UiPlayStore.Devices.filter(d => d.DeviceID !== device.DeviceID);
                    }}>
                    <Trash size={20} />
                  </button>
                }
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
