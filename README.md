# Tauri Htmx Demo Application

Due to platform limitations, various network calls are patched and multiple components handle different parts of the
app's request lifecycle.

Here's an overview:

1. Application opens `proxy://localhost` (`proxy` scheme). The `proxy` scheme is used by default, and primarily used to
   serve assets and other requests that doesn't require body (Android don't support reading body with custom schemes).
2. Htmx calls are intercepted to call the Tauri command `proxy_request`, using a script (see `interceptor.js`) injected
   by the backend server.
   This external script should be injected locally through Tauri in the future (using plugin, or other means to inject
   window js).
3. As redirects aren't supported with custom schemes, they are captured inside Tauri scheme and command handlers, and an
   inline js code `eval`d to emulate the behavior.

## Running

To build and install the application on your Android device, you can use the following commands:

```shell
cd src-tauri
# Only required once if Android wasn't initialized yet.
cargo tauri android init

# Replace the ip here as required, to match your computer.
# Debug builds are too big, takes long time to transfer and install to device, so we use release build.
# Make sure your device is connected with `adb connect` and present in `adb devices`.
TAURI_PROXY_BASE_URL=http://192.168.1.110:8003 cargo tauri android build --target aarch64 --apk \
&& adb install "$PWD/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
```

## RoadMap

There are still lots to do! Some points we need to address:

TODO:

1. Make the "solution" a Tauri plugin and inject the htmx interceptor on all pages.
2. Persist cookies on an encrypted reliable storage, db plugin?
3. Improve error handling, expose more details to the user for reporting purposes, but without leaking internals.
4. Disable browser navigation. "Back button" breaks the app.
5. Sometimes, it closes when you open the app back again from the Android task switcher. Tauri/webview related?
6. Multipart forms and other bodies don't work, we need to implement this. Is it possible to get the Multipart data from
   the browser?
7. Keyboard doesn't resize and shifts the view on Android, but covers the page and form inputs instead.
8. Implement async scheme handler.
9. Implement at least basic asset caching, it is being slower even on local network.
10. Loading indicators, probably at Tauri level.
11. Test on iOS.
12. Disable "text selection" and other browser-like behavior on the app.

Considerations:

1. We'll need to implement websockets, probably using another async scheme handler.
2. We only cover htmx now, any new network communications will need to be captured explicitly (let's keep in mind).
3. App is rather big by now? Release build weights ~25 MB (debug ~425MB!)?

## What Didn't Work?

To avoid this approach, we tried various other candidates to no avail.

1. Bundle http server:
    - Bundle a http server, like Actix.
    - Needs local tls, but we can't provide private key securely. Also, possible self-signed cert issues on mobile.
2. Service workers:
    - Use service workers and listen `fetch` event to intercept network traffic.
    - Needs to load the service worker definition from `http/s` per spec.
3. Bare interception:
    - Intercept network/htmx calls and translate to Tauri calls, without doing anything else.
    - Didn't work as we still need to proxy assets.
4. Bare custom scheme:
    - Implement a custom scheme to proxy requests, without doing anything else.
    - Didn't work due to Android limitation; we can't get request body.

So, we ended up combining "solutions" `3` and `4`, intercept htmx calls and use a custom scheme by default.
