# Release

Build the release from the final verified commit, never earlier: one build
was bundled before its last fixes and had to be rebuilt.

## Order

1. `gate.sh` → `gate: PASS`; the verification report is written; the
   working tree is clean and committed.
2. Icon: `tools/make-icon.py assets/icon/icon_1024.png`; look at it at 1024
   and 64 px.
3. Bundle: `tools/bundle-macos.sh`
   - universal when the rustup toolchain has both `aarch64-apple-darwin` and
     `x86_64-apple-darwin` (both slices with the same compiler, then `lipo`);
     otherwise host only (`--host-only` forces it) and say so;
   - builds in the shared folder `target/shared` with a disk guard;
   - `Info.plist`: name, display name, bundle id `<prefix>.<slug>`, version
     `<x.y.z>.<git sha>`, `LSMinimumSystemVersion` 13.0, category, high
     resolution, copyright line naming the font licence; usage strings only
     for permissions the app really uses (local network for sync, microphone);
   - licences inside `Contents/Resources`: the app licence, Geist OFL, every
     other bundled font or asset licence;
   - signing: a Developer ID Application identity (found with `security
     find-identity -v -p codesigning`, or `<SLUG>_SIGN_IDENTITY`) with
     hardened runtime and secure timestamp, binary first, then the bundle;
     else ad-hoc (`--adhoc`), and the report says other Macs will warn;
   - `codesign --verify --deep --strict`, `lipo -archs`.
4. Launch the bundle once, in background mode with a throwaway data folder,
   and stop it by its pid (never by name: the user may run their own copy):
   ```bash
   D="target/bundle-check"; mkdir -p "$D"
   <SLUG>_NO_AUDIO=1 "dist/<Name>.app/Contents/MacOS/<slug>" --data-dir "$D" --trace --background &
   pid=$!; sleep 3; grep -m1 '"kind":"start"' "$D/trace.jsonl"; kill "$pid"
   ```
5. Install where the learner practises. On this Mac:
   `tools/install-local.sh` (refuses while a `<slug>` process runs, moves an
   old copy to the Trash, `ditto` into `~/Applications`, checks signature,
   architectures and `--version`). On another Mac (allowance from round 1):
   `tools/install-remote.sh <ssh-host>`. It refuses while the app runs
   there, moves the old app to that Mac's Trash, copies with rsync, and
   checks signature, architectures and `--version` remotely. The learner's
   data folder is never touched. Copies over ssh carry no quarantine flag, so
   an unnotarized app opens there.
6. Notarization (only with a fresh yes; it uploads the app to Apple):
   `<SLUG>_NOTARY_PROFILE=<profile> tools/notarize-macos.sh` → submit, wait,
   staple, `spctl -a -vv` must say "accepted, Notarized Developer ID", then a
   shareable zip. Look for an existing keychain profile first; if none,
   the user creates one with `xcrun notarytool store-credentials` (never ask
   for an app password in chat).
7. Sharing: the zip contains the app only. Licensed packs and private
   material never go in. Friends add their own assets through the app.

## Licences

- The app: the user's choice (personal: all rights reserved; public: a
  licence file). A GPL dependency (for example Ableton Link) makes the app
  GPL: ask in the grilling, ship the licence text, and say "anyone who gets
  the app may ask for the source".
- Fonts: Geist is SIL OFL 1.1; ship `OFL.txt` in the bundle.
- Content: own transcriptions under generic names; no third-party MIDI,
  samples or clips in git or the bundle unless the licence allows it.
- Licensed sample libraries: personal use, copied (never moved or changed)
  into the app's data folder on the user's own Macs only.

## Architecture claims

State which architectures were actually run: "arm64 run on the build Mac;
x86_64 started under Rosetta only" is honest; "universal, tested" is not
unless an Intel Mac ran it.

## README of the app

First line: one sentence of purpose. Then: schedule (none), output (the data
folder), how to turn off and remove (move the app and the data folder to the
Trash), how to run, check (`gate.sh`), verify (`/verify-<slug>`), bundle,
notarize (with the owner's OK), install.
