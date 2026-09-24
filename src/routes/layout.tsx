import {
  component$,
  createContextId,
  isBrowser,
  Signal,
  Slot,
  useContextProvider,
  useSignal,
  useStore,
  useVisibleTask$,
} from '@qwik.dev/core';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import Nav from '~/components/Nav';
import { listenToUxPlayOutput } from '~/util/listen';
import { UiPlayStoreType } from '~/util/types';

export const UiPlayStoreContext =
  createContextId<UiPlayStoreType>('UiPlayStoreContext');
export const TerminalRefContext =
  createContextId<Signal<HTMLDivElement>>('TerminalRefContext');
export default component$(() => {
  const UiPlayStore = useStore<UiPlayStoreType>({
    Settings: {
      Name: 'UiPlay',
      Provider: 'shairport',
    },
    Devices: [],
  });
  useContextProvider(UiPlayStoreContext, UiPlayStore);

  const terminalRef = useSignal<HTMLDivElement>();
  useContextProvider(TerminalRefContext, terminalRef);
  const fitSignal = useSignal<FitAddon>();
  const terminalInstance = useSignal<Terminal>();

  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async () => {
    void invoke<UiPlayStoreType['Settings']>('get_settings').then(
      (settings) => {
        UiPlayStore.Settings = settings;
      }
    );

    if (!terminalRef.value || !isBrowser) return;

    // get current css variable since xterm doesn't support variables directly
    const fgColor =
      getComputedStyle(document.documentElement)
        .getPropertyValue('--color-lum-text')
        .trim() || '#ffffff';
    const term = new Terminal({
      cursorBlink: true,
      macOptionIsMeta: true,
      scrollback: 1000,
      theme: {
        background: '#00000000',
        foreground: fgColor,
      },
    });
    const webLinks = new WebLinksAddon();
    const search = new SearchAddon();
    const fitAddon = new FitAddon();

    terminalInstance.value = term;
    fitSignal.value = fitAddon;

    await Promise.all([
      ...['Title', 'Artist', 'Album', 'Genre'].map((eventName) =>
        listen<string>(eventName, (event) => {
          UiPlayStore.NowPlaying = {
            ...UiPlayStore.NowPlaying,
            [eventName]: event.payload,
          };
        })
      ),
      listen<string>('uxplay-output', (event) => {
        term.write(event.payload + '\r\n');
        listenToUxPlayOutput(event, UiPlayStore);
      }),
      listen<string>('app-output', (event) => {
        term.write(event.payload + '\r\n');
      }),
    ]);

    term.loadAddon(fitSignal.value);
    term.loadAddon(webLinks);
    term.loadAddon(search);

    term.open(terminalRef.value);
    requestAnimationFrame(() => {
      fitAddon.fit();
      term.focus();
    });

    // Set up resize observer to handle container size changes
    const resizeObserver = new ResizeObserver(() => {
      if (fitSignal.value) {
        fitSignal.value.fit();
      }
    });

    resizeObserver.observe(terminalRef.value);
  });

  return (
    <>
      <Nav />

      <div
        class={{
          'rounded-lum-2 fixed inset-0 -z-10 h-full w-full blur-[5vmax] saturate-200 transition-opacity duration-10000': true,
          'opacity-0': !UiPlayStore.NowPlaying?.AlbumArt,
          'opacity-20': UiPlayStore.NowPlaying?.AlbumArt,
        }}
      >
        <img
          class="animate-spin-cc anim-duration-40000 absolute top-0 right-0 h-[50vmax] w-[50vmax] -translate-x-1/3 -translate-y-1/3 scale-150 rounded-xl object-cover"
          src={UiPlayStore?.NowPlaying?.AlbumArt}
          alt={UiPlayStore?.NowPlaying?.Album}

          width={1024}
          height={1024}
        />
        <img
          class="anim-duration-40000 absolute top-0 left-0 h-[50vmax] w-[50vmax] translate-x-1/3 translate-y-1/3 scale-150 animate-spin rounded-xl object-cover"
          src={UiPlayStore?.NowPlaying?.AlbumArt}
          alt={UiPlayStore?.NowPlaying?.Album}

          width={1024}
          height={1024}
        />
        <img
          class="animate-spin-cc anim-duration-40000 absolute bottom-0 left-0 h-[50vmax] w-[50vmax] -translate-x-1/3 -translate-y-1/3 scale-150 rounded-xl object-cover"
          src={UiPlayStore?.NowPlaying?.AlbumArt}
          alt={UiPlayStore?.NowPlaying?.Album}

          width={1024}
          height={1024}
        />
        <img
          class="anim-duration-40000 absolute right-0 bottom-0 h-[50vmax] w-[50vmax] translate-x-1/3 translate-y-1/3 scale-150 animate-spin rounded-xl object-cover"
          src={UiPlayStore?.NowPlaying?.AlbumArt}
          alt={UiPlayStore?.NowPlaying?.Album}

          width={1024}
          height={1024}
        />
      </div>

      <main class="flex min-h-screen flex-col justify-center px-16 lg:mx-[10vw]">
        <Slot />
      </main>

      <div
        id="terminal-container"
        class={{
          'lum-bg-bg rounded-lum absolute inset-24 p-4 backdrop-blur-lg transition-all duration-300': true,
          'pointer-events-none -mt-2 opacity-0': !UiPlayStore.TerminalOpen,
        }}
      >
        <div ref={terminalRef} class="h-full w-full overflow-hidden" />
      </div>
    </>
  );
});
