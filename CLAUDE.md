# Altoid - Desktop Camera Upload Client

## Dev Architecture

### Languages
- **TypeScript** — Frontend UI (React components, hooks, services, types)
- **Rust** — Backend desktop runtime (Tauri commands, camera detection, HTTP/OAuth, state management)
- **CSS** — Styling

### Frameworks & Key Libraries

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

### Local Setup

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

# Terminal 2: Run Rust backend via IDE run config (see README for screenshot)
```

**Build for production:**
```bash
yarn run tauri build
```

### Project Structure
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

---

## Coding Rules

### Frontend (TypeScript/React)

- **Functional components only** — no class components. Use `export default function ComponentName()`.
- **React Query for all server/backend state** — queries for reads, mutations for writes. No raw `useEffect` for data fetching.
  - Define query keys as exported constants: `export const USER_QUERY_KEY = ['user'] as const;`
  - Set `staleTime` and `gcTime` explicitly per query.
  - Invalidate or set query data in mutation `onSuccess` callbacks.
- **Service layer for IPC** — never call `invoke()` directly from components. Use service functions in `src/contexts/services/` which wrap `invoke()` with timeouts and error handling.
- **Timeout wrapper pattern** — all IPC calls go through `withTimeout()` to prevent hangs.
- **Props as inline types** — define props inline on the function signature: `function Header({ onOpenSettings }: { onOpenSettings: () => void })`.
- **Export shared types from components** — types like `UploadStatus` and `UploadEntry` are exported from the component that owns them.
- **Rust-mirrored types in `src/rust-api/model/`** — keep TypeScript interfaces in sync with Rust structs. Serde renames (`snake_case` -> `camelCase`) are handled via `#[serde(rename)]`.
- **File naming**: PascalCase for components (`.tsx`), camelCase for hooks/services (`.ts`), `use[Name]Query.ts` / `use[Name]Mutation.ts` for hooks.

### Backend (Rust)

- **`AppError` for all errors** — use the `AppError` enum with `thiserror`. Use helper constructors (`AppError::auth_failed()`, `AppError::internal()`, `AppError::camera_op()`) for ergonomic creation.
- **Two-layer error handling at IPC boundary** — internal functions return `Result<T, AppError>`. Tauri commands convert errors to `IpcResponse` JSON via `err_response()` before returning to the frontend.
- **Global state via `OnceLock<AppState>`** — initialized once in Tauri `setup`. Access with `APP_STATE.get()`. Mutex-protected interior fields.
- **`LazyLock` for singletons** — HTTP client and other long-lived resources use `LazyLock` for lazy initialization.
- **Persistence as JSON files** — config and uploaded-files state are saved to the app's local data directory (`altoid_config.json`, `uploaded_files.json`). No database.
- **Public model files prefixed `pub_`** — e.g., `pub_user_info.rs`, `pub_ipc_response.rs` for types that cross the IPC boundary.
- **Tauri event emissions for progress** — long-running operations (file uploads) emit events (`file-progress`) that the frontend listens to, rather than returning intermediate results.
- **`reqwest` with bearer tokens** — all API requests include `Authorization: {token_type} {access_token}` header. 30-second timeout default.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/enums, `UPPER_SNAKE_CASE` for constants.
