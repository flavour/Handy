# Running

```bash
sudo apt install build-essential libasound2-dev pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf cmake
bun install
bun run tauri dev
```

Use Default mic to go via Pipewire, do NOT go direct to the Webcam.
Ctrl+Shift+D to enable Debug Panel on UI

# Building

```bash
bun run tauri build --bundles deb
cd src-tauri/target/release/bundle/deb
sudo apt install ./Handy*.deb
```

## Known Failure Modes / Fixes

### Vulkan shader build fails (we don't need Vulkan)

Symptom:

- Build fails in `whisper-rs-sys` with errors like:
  - `ggml-vulkan.cpp: error: 'timestep_embedding_f32_data' was not declared in this scope`

Cause:

- On Linux in this workspace, `transcribe-rs v0.2.2` pulls in `whisper-rs` with feature `vulkan` enabled (it is hard-coded in `transcribe-rs`'s `cfg(target_os = "linux")` dependency declaration).
- That enables `GGML_VULKAN=ON` in the underlying ggml build.
- On Linux, ggml's Vulkan backend expects `glslc` to generate `ggml-vulkan-shaders.hpp`.
- If `glslc` is missing, shader generation produces an empty header, so `*_data` / `*_len` symbols are missing and compilation fails.

Fix options:

1. Prefer (CPU-only): remove the `whisper-rs/vulkan` requirement.
   - Confirm the source:
     ```bash
     cargo tree -e features -i whisper-rs --manifest-path src-tauri/Cargo.toml | rg -n "vulkan|transcribe-rs"
     ```
   - Because `transcribe-rs` hard-codes `whisper-rs` with `features = ["vulkan"]` on Linux, the practical way to do this is to patch `transcribe-rs`:
     - Implemented in this workspace by vendoring and patching:
       - Vendor path: `src-tauri/vendor/transcribe-rs`
       - Patched dependency: removed `features = ["vulkan"]` from `src-tauri/vendor/transcribe-rs/Cargo.toml`
       - Override via `[patch.crates-io]` in `src-tauri/Cargo.toml`:
         - `transcribe-rs = { path = "vendor/transcribe-rs" }`
     - Then rebuild from a clean slate:
       ```bash
       rm -rf src-tauri/target
       cargo build --manifest-path src-tauri/Cargo.toml -p handy
       ```

2. If you _do_ want Vulkan: install `glslc` so shader generation works.
   - Note: on Ubuntu 22.04 in this environment, `apt-cache search glslc` did not return a package. If you need Vulkan, install a Vulkan SDK/toolchain that provides `glslc` (or add a repo that ships Shaderc tools).

### X11 crash when starting transcription (xcb/xlib threading)

Symptom:

- When starting transcription/recording, the process aborts with logs like:
  - `[xcb] Most likely this is a multi-threaded client and XInitThreads has not been called`
  - `Assertion '!xcb_xlib_threads_sequence_lost' failed.`

Fix:

- Call `XInitThreads()` at process start on Linux/X11 before any Xlib usage.
- Implemented in `src-tauri/src/main.rs` (guarded by `DISPLAY`).

### Jack

I removed the JACK option because it wasn’t actually supported by the CPAL version in this repo, and it would fail to compile.
- I initially added HANDY_CPAL_HOST=jack with cpal::HostId::Jack.
- But in your build, cpal::HostId does not have a Jack variant, so that code path is invalid at compile time (not a runtime problem, a “this enum variant doesn’t exist” problem).
- So I pared it back to default (whatever CPAL picks) and alsa (explicit), which are both valid here.
If you really want a JACK backend option later, we’d need to confirm CPAL can be built with a JACK host on Linux in this project (feature flags / version), and then re-add it in a way that compiles conditionally.

