import { ssgAdapter } from "@qwik.dev/router/adapters/ssg/vite";
import { extendConfig } from "@qwik.dev/router/vite";
import baseConfig from "../../vite.config.ts";

export default extendConfig({
  ...baseConfig,
  // typeAware and typeCheck are not allowed in nested vite configs
  lint: {
    ...baseConfig.lint,
    options: {
      ...baseConfig.lint?.options,
      typeAware: undefined,
      typeCheck: undefined
    }
  }
}, () => {
  return {
    build: {
      ssr: true,
      rolldownOptions: {
        input: ["@qwik-router-config"],
      },
    },
    plugins: [
      ssgAdapter({
        origin: "http://127.0.0.1:5173",
      }),
    ],
  };
});
