# FilmCraft for Android

`package.sh` builds `filmcraft-<version>-android-arm64.apk` (Android 8.0+, 64-bit ARM) from
[`apps/filmcraft-android`](../../apps/filmcraft-android): the same engine and editor UI as the
desktop app on a NativeActivity (Vulkan, GLES fallback; AAudio playback through cpal).

CI: [`.github/workflows/release-windows-android.yml`](../../.github/workflows/release-windows-android.yml)
builds it with the Windows packages and attaches both to a GitHub release (push a `v*` tag or run
the workflow by hand).

## Media on the phone

Android gives a native activity no file dialog, so FilmCraft uses its folder on shared storage,
`Android/data/ai.storyteller.filmcraft/files/` (reachable over USB and from most file managers,
no permission needed):

| Folder | Used by |
|---|---|
| `Import/` | File ▸ Import adds every matching file in it; relinking picks from it |
| `Projects/` | Save / Save As; Open takes the most recently changed project |
| `Exports/` | exports and exported frames |

Auto-save, crash recovery, preferences and logs live in the app's internal storage
(`adb logcat -s filmcraft` shows the log).

## Signing

Set `ANDROID_KEYSTORE_BASE64` (`base64 -w0 release.keystore`) and `ANDROID_KEYSTORE_PASSWORD` as
repository secrets so every release is signed with the same key; otherwise each build gets a
throwaway key and a new APK cannot install over the previous one (uninstall it first).

## Known limits

- The interface is the desktop one: it is usable on a tablet or a large phone in landscape, with
  touch acting as the mouse; there is no phone-specific layout yet.
- No hardware video decoding (the pure-Rust decoders are used), no voice-over recording.
