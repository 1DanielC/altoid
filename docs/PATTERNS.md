# Architecture Patterns

## Frontend (TypeScript/React)

- **React Query for all server/backend state** — queries for reads, mutations for writes. No raw `useEffect` for data fetching.
  - Define query keys as exported constants: `export const USER_QUERY_KEY = ['user'] as const;`
  - Set `staleTime` and `gcTime` explicitly per query.
  - Invalidate or set query data in mutation `onSuccess` callbacks.
- **Service layer for IPC** — never call `invoke()` directly from components. Use service functions in `src/contexts/services/` which wrap `invoke()` with timeouts and error handling.
- **Timeout wrapper pattern** — all IPC calls go through `withTimeout()` to prevent hangs.

## Backend (Rust)

- **`AppError` for all errors** — use the `AppError` enum with `thiserror`. Use helper constructors (`AppError::auth_failed()`, `AppError::internal()`, `AppError::camera_op()`) for ergonomic creation.
- **Two-layer error handling at IPC boundary** — internal functions return `Result<T, AppError>`. Tauri commands convert errors to `IpcResponse` JSON via `err_response()` before returning to the frontend.
- **Global state via `OnceLock<AppState>`** — initialized once in Tauri `setup`. Access with `APP_STATE.get()`. Mutex-protected interior fields.
- **`LazyLock` for singletons** — HTTP client and other long-lived resources use `LazyLock` for lazy initialization.
- **Persistence as JSON files** — config and uploaded-files state are saved to the app's local data directory (`altoid_config.json`, `uploaded_files.json`). No database.
- **Tauri event emissions for progress** — long-running operations (file uploads) emit events (`file-progress`) that the frontend listens to, rather than returning intermediate results.
- **`reqwest` with bearer tokens** — all API requests include `Authorization: {token_type} {access_token}` header. 30-second timeout default.
- **Static Strings** — Prefer static strings over hardcoded inline strings.
  - Bad: `api.get_file("file.txt").await?["content"]`
  - Good: `api.get_file(FILE_ID).await?["content"]`
