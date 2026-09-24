import { component$, useContext } from '@qwik.dev/core';
import type { DocumentHead } from '@qwik.dev/router';
import { UiPlayStoreContext } from '../layout';
import Save from 'lucide-icons-qwik/icons/Save';
import { invoke } from '@tauri-apps/api/core';

export default component$(() => {
  const UiPlayStore = useContext(UiPlayStoreContext);

  return (
    <>
      <h1 class="text-2xl font-bold">Settings</h1>
      <p class="mb-4 text-lg">Configure your preferences here.</p>

      <label for="name" class="mb-1 font-medium text-gray-400">
        Name
      </label>
      <input
        id="name"
        type="text"
        class="lum-input"
        placeholder="Enter the AirPlay receiver name"
        value={UiPlayStore.Settings.Name}
        onInput$={(e, el) => (UiPlayStore.Settings.Name = el.value)}
      />

      <label for="provider" class="mt-4 mb-1 font-medium text-gray-400">
        Provider
      </label>
      <select
        id="provider"
        class="lum-input"
        value={UiPlayStore.Settings.Provider}
        onChange$={(_, el) => {
          UiPlayStore.Settings.Provider = el.value as 'shairport' | 'uxplay';
        }}
      >
        <option value="shairport">Shairport Sync (recommended)</option>
        <option value="uxplay">UxPlay</option>
      </select>

      <div>
        <button
          class="lum-btn lum-bg-green-800 mt-4"
          onClick$={async () => {
            await invoke('save_settings', { settings: UiPlayStore.Settings });
            void invoke('start_mediaplayer');
          }}
        >
          <Save size={24} />
          Save and restart receiver
        </button>
      </div>
    </>
  );
});

export const head: DocumentHead = {
  title: 'UiPlay',
  meta: [
    {
      name: 'description',
      content: 'Configure UiPlay.',
    },
  ],
};
