import { component$, useContext } from '@qwik.dev/core';
import type { DocumentHead } from '@qwik.dev/router';
import { UiPlayStoreContext } from '../layout';
import Save from 'lucide-icons-qwik/icons/Save';

export default component$(() => {
  const UiPlayStore = useContext(UiPlayStoreContext);

  return (
    <>
      <h1 class="text-2xl font-bold">Settings</h1>
      <p class="text-lg mb-4">Configure your preferences here.</p>

      <label for="name" class="font-medium text-gray-400 mb-1">Name</label>
      <input
        id="name"
        type="text"
        class="lum-input"
        placeholder="Enter the name of the UxPlay instance"
        value={UiPlayStore.Settings.Name}
        onInput$={(e, el) => UiPlayStore.Settings.Name = el.value }
      />

      <div>
        <button
          class="lum-btn mt-4 lum-bg-green-800"
          onClick$={() => {
            // Logic to save settings can be added here
            console.log('Settings saved:', UiPlayStore.Settings);
          }}
        >
          <Save size={24} />
          Save and restart UxPlay
        </button>
      </div>
      <div class="mt-6">
        <h2 class="text-lg font-semibold">Current Settings</h2>
        <pre class="bg-gray-800 p-4 rounded-lg">
          {JSON.stringify(UiPlayStore.Settings, null, 2)}
        </pre>
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
