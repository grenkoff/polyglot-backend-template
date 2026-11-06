import { defineConfig, enforceTdd } from '@nizos/probity'

export default defineConfig({
  rules: [
    {
      files: [
        'services/*/src/**/*.rs',
        'tools/*/src/**/*.rs',
        'services/core/server/**/*.py',
        '!services/core/server/contracts/**',
        '!services/core/server/settings/**',
        '!services/core/server/**/migrations/**',
        '!**/main.rs',
      ],
      rules: [enforceTdd()],
    },
  ],
})
