const { withNativeFederation, shareAll } = require('@angular-architects/native-federation/config');

module.exports = withNativeFederation({
  name: 'aad',



  exposes: {
    './routes': './src/app/app.routes.ts',
    './Component': './src/app/app.component.ts',
  },

  shared: {
    ...shareAll({ singleton: true, strictVersion: false, requiredVersion: 'auto' }),
  },

  skip: [
    'rxjs/ajax',
    'rxjs/fetch',
    'rxjs/testing',
    'rxjs/webSocket',
    '@fontsource/inter',
    'material-symbols',
    '@softarc/native-federation-node',
    'autoprefixer',
    'tailwindcss',
    'glob-parent',
    /^marked/,
    '@tailwindcss/container-queries',
    '@tailwindcss/forms',
    '@tailwindcss/typography',
    // Add further packages you don't need at runtime
  ],

  // Please read our FAQ about sharing libs:
  // https://shorturl.at/jmzH0

  features: {
    // New feature for more performance and avoiding
    // issues with node libs. Comment this out to
    // get the traditional behavior:
    ignoreUnusedDeps: true
  }
});
