import { component$, useContext } from '@qwik.dev/core';
import { Link } from '@qwik.dev/router';
import { Window } from '@tauri-apps/api/window';
import {
  Airplay,
  Minus,
  Settings,
  Square,
  Terminal,
  X,
} from 'lucide-icons-qwik';
import { UiPlayStoreContext } from '~/routes/layout';
import { Nav } from '@luminescent/ui-qwik';

export default component$(() => {
  const UiPlayStore = useContext(UiPlayStoreContext);

  return (
    <Nav
      fixed
      data-tauri-drag-region
      innerProps={
        {
          'data-tauri-drag-region': true,
          style: {
            '--lum-depth': '1',
          },
        } as Record<string, unknown>
      }
      colorClass="lum-grad-bg-gray-800/20"
      style={{
        '--lum-border-radius': '1rem',
      }}
      nohamburger
    >
      <Link
        q:slot="start"
        href="/"
        class="lum-btn lum-bg-transparent rounded-lum-2 text-sm font-semibold"
      >
        <Airplay size={16} strokeWidth={3} />
        UiPlay
      </Link>

      <Link
        q:slot="end"
        href="/settings"
        class="lum-btn lum-bg-transparent rounded-lum-2 p-2"
      >
        <Settings size={16} />
      </Link>
      <button
        q:slot="end"
        class="lum-btn lum-bg-transparent rounded-lum-2 p-2"
        onClick$={() => {
          UiPlayStore.TerminalOpen = !UiPlayStore.TerminalOpen;
        }}
      >
        <Terminal size={16} />
      </button>

      <button
        q:slot="end"
        class="lum-btn lum-bg-transparent rounded-lum-2 p-2"
        onClick$={async () => await Window.getCurrent().minimize()}
      >
        <Minus size={16} />
      </button>
      <button
        q:slot="end"
        class="lum-btn lum-bg-transparent rounded-lum-2 p-2"
        onClick$={async () => await Window.getCurrent().toggleMaximize()}
      >
        <Square size={16} />
      </button>
      <button
        q:slot="end"
        class="lum-btn lum-bg-transparent rounded-lum-2 hover:lum-bg-red p-2"
        onClick$={async () => await Window.getCurrent().close()}
      >
        <X size={16} />
      </button>
    </Nav>
  );
});
