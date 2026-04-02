# Naming & File Conventions

## Frontend (TypeScript/React)

- **File naming**: PascalCase for components (`.tsx`), camelCase for hooks/services (`.ts`), `use[Name]Query.ts` / `use[Name]Mutation.ts` for hooks.
- **Functional components only** — no class components. Use `export default function ComponentName()`.
- **Props as inline types** — define props inline on the function signature: `function Header({ onOpenSettings }: { onOpenSettings: () => void })`.
- **Export shared types from components** — types like `UploadStatus` and `UploadEntry` are exported from the component that owns them.
- **Rust-mirrored types in `src/rust-api/model/`** — keep TypeScript interfaces in sync with Rust structs. Serde renames (`snake_case` -> `camelCase`) are handled via `#[serde(rename)]`.

## Backend (Rust)

- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/enums, `UPPER_SNAKE_CASE` for constants.
- **Public model files prefixed `pub_`** — e.g., `pub_user_info.rs`, `pub_ipc_response.rs` for types that cross the IPC boundary.
