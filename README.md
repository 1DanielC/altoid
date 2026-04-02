# Desktop Sync Client
Sometimes, users taking large captures may have a difficult time uploading their captures from their
phones, when the video files are GB's in size. This application allows users to upload their
files from their desktop, where internet connecitivity is better, and more disk space is available.

## Languages
- **TypeScript** — Frontend UI (React components, hooks, services, types)
- **Rust** — Backend desktop runtime (Tauri commands, camera detection, HTTP/OAuth, state management)
- **CSS** — Styling

## Frameworks & Key Libraries

**Frontend:**
- **React 19** — UI framework (functional components only)
- **Vite 5** — Dev server and bundler
- **TanStack React Query 5** — Data fetching, caching, and mutations
- **Tauri API v2** — IPC bridge to Rust backend (`@tauri-apps/api`)
- **Tauri Plugins** — File system (`plugin-fs`), opener (`plugin-opener`), logging (`plugin-log`)

**Backend (Rust):**
- **Tauri 2** — Desktop app framework, IPC command layer
- **Tokio** — Async runtime (multi-threaded)
- **Reqwest** — HTTP client (rustls-tls, no OpenSSL dependency)
- **Serde / serde_json** — JSON serialization
- **Rusb** — USB device detection
- **gphoto2-sys** — PTP camera protocol support
- **bb-drivelist** — Drive enumeration
- **thiserror** — Ergonomic error type derivation

## Local Setup

**Prerequisites:**
- Node.js + Yarn
- Rust toolchain (rustup)
- Tauri CLI (`cargo install tauri-cli`)

**Run the full app (terminal):**
```bash
yarn install
yarn run tauri dev
```

**Run frontend + backend separately (IDE debugging):**
```bash
# Terminal 1: Frontend dev server (port 1420)
yarn dev

# Terminal 2: Run Rust backend via IDE run config
```
![img.png](img.png)

**Build for production:**
```bash
yarn run tauri build
```

## Project Structure
```
src/                          # Frontend (React/TypeScript)
├── components/               # UI components (Header, Footer, LoginButton, UploadTable, etc.)
├── contexts/                 # React context providers
│   └── services/             # Service layer wrapping Tauri IPC (ApiService, CameraService, SystemService)
├── hooks/
│   ├── queries/              # React Query hooks (useUserQuery, useCameraQuery)
│   └── mutations/            # React Query mutations (useLoginMutation, useLogoutMutation, etc.)
├── config/                   # Query client config
├── rust-api/model/           # TypeScript types mirroring Rust structs
├── services/                 # Utility services (logging)
└── App.tsx                   # Root component

src-tauri/                    # Backend (Rust/Tauri)
├── src/
│   ├── lib.rs                # Tauri command handlers (IPC entry points)
│   ├── main.rs               # App entry point
│   ├── state.rs              # AppState, config persistence (JSON files)
│   ├── error.rs              # AppError enum (thiserror)
│   ├── api/
│   │   ├── oauth/            # Device code OAuth flow
│   │   ├── openspace/        # OpenSpace REST API client
│   │   └── http/             # HTTP client setup
│   ├── camera/               # USB + PTP camera detection and file listing
│   ├── ipc/                  # IPC response/request types
│   └── traits/               # Shared traits (ToJson, OptionExt, ResultExt)
├── Cargo.toml
└── tauri.conf.json
```
