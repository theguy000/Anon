# Feature Implementation Plan — Anon Browser

## Overview

13 new features for the Anon Instance Manager, organized into 4 implementation phases:
- **Phase 1: Core UX** — Rename, Search/Filter, Toast Notifications, Dedicated Settings Page
- **Phase 2: Instance Management** — Tags, Notes, Bulk Operations, Import/Export
- **Phase 3: Proxy & Fingerprint** — Proxy Testing, Proxy Rotation, Fingerprint Rotation, Fingerprint Test Sites
- **Phase 4: Data Management** — Session/Cookie Viewer

---

## Phase 1: Core UX Improvements

### 1.1 Instance Renaming (Inline Editing)

**Behavior:** Double-click the instance name on InstanceCard (both grid and list views) to enter edit mode. Press Enter to save, Escape to cancel. Validate uniqueness (case-insensitive) before saving.

**Backend (`src-tauri/src/instances.rs`):**
- Add new command `rename_instance(app: AppHandle, id: String, new_name: String) -> Result<(), String>`
- Validate: instance exists, name not empty, name not duplicate (case-insensitive), instance not running
- Update `anon_config.json` with new name

**Backend (`src-tauri/src/lib.rs`):**
- Register `rename_instance` in `tauri::generate_handler![]`

**Frontend (`src/lib/store.ts`):**
- Add `renameInstance(id: string, newName: string): Promise<void>` — calls `rename_instance`, reloads instances

**Frontend (`src/lib/components/instance/InstanceCard.svelte`):**
- Add state: `editing = false`, `editName = ''`, `editError = ''`
- On double-click name: set `editing = true`, `editName = instance.name`
- Render: if `editing`, show `<input>` with `editName` bound, auto-focused. Show `editError` inline.
- On Enter: validate locally (non-empty, no duplicate in `$instances`), call `renameInstance()`, catch errors. On success: `editing = false`.
- On Escape/blur: `editing = false`
- Prevent editing when instance is running

### 1.2 Search & Filter Bar

**Full filter bar** with: name search, status filter (all/running/stopped), tag filter (after 2.1), proxy type filter (all/none/http/socks4/socks5), persistence filter (all/on/off).

**Frontend (`src/routes/+page.svelte`):**
- Add state: `searchQuery = ''`, `statusFilter = 'all'`, `proxyFilter = 'all'`, `persistFilter = 'all'`, `tagFilter = 'all'`
- Add reactive `filteredInstances` derived from `sortedInstances`:
  ```
  sortedInstances
    → filter by name (case-insensitive includes)
    → filter by running status
    → filter by proxy_config.proxy_type
    → filter by persist_data
    → filter by tag (Phase 2)
  ```
- Render: filter bar above instance list with:
  - Text input (search icon + clear button)
  - Status dropdown: ALL / RUNNING / STOPPED
  - Proxy dropdown: ALL / NONE / HTTP / SOCKS4 / SOCKS5
  - Persist dropdown: ALL / ON / OFF
  - Tag dropdown (placeholder, populated in Phase 2)
  - Instance count badge: "X of Y instances"
- Use `filteredInstances` instead of `sortedInstances` in the `{#each}` loop

### 1.3 Toast Notification System

**Frontend — New component (`src/lib/components/ui/Toast.svelte`):**
- Reactive toast container (fixed, bottom-right, z-index: 2000)
- Each toast: message, type (success/error/warning/info), auto-dismiss timer (3s default, 5s for errors)
- Animations: slide-in from right, fade-out
- Styling: matches existing dark theme, colored left border accent per type
  - success: `--accent-running` (#00ff41)
  - error: `--accent-danger` (#ff4444)
  - warning: `--accent-warning` (#ff8800)
  - info: white border

**Frontend — New store (`src/lib/store.ts`):**
- Add `toasts` writable store: `Array<{ id: string, message: string, type: 'success'|'error'|'warning'|'info', duration: number }>`
- Add `addToast(message, type, duration?)` — pushes to store, auto-removes after duration
- Add `removeToast(id)` — manual dismiss

**Integration points (update existing functions in `store.ts`):**
- `createInstance` → success toast "Instance created"
- `deleteInstance` → success toast "Instance deleted"
- `renameInstance` → success toast "Instance renamed"
- `launchInstance` → success toast "Instance launched" / error toast on failure
- `stopInstance` → success toast "Instance stopped"
- `updateInstanceSettings` → success toast "Settings saved" / warning toast for conflicts
- `updateInstanceProxy` → success toast "Proxy saved"
- `togglePersistence` → success toast
- All `catch` blocks → error toast with message

**Frontend (`src/routes/+layout.svelte`):**
- Add `<Toast />` component at the layout level (visible globally)

### 1.4 Dedicated Settings Page

**Approach:** Since the app is single-page, implement as a modal/panel rather than a route. Add a settings gear icon in the header (layout.svelte).

**Backend (`src-tauri/src/settings.rs`):**
- Expand `AppSettings` struct:
  ```rust
  pub struct AppSettings {
      pub skip_wipe_confirmation: bool,
      pub skip_delete_confirmation: bool,
      // New fields:
      pub default_view_mode: Option<String>,         // "grid" | "list"
      pub default_sort_field: Option<String>,         // "name" | "created_at"
      pub default_sort_dir: Option<String>,           // "asc" | "desc"
      pub default_proxy_template: Option<ProxyConfig>, // default proxy for new instances
      pub default_fingerprint_template: Option<FingerprintConfig>, // default fingerprint
      pub camoufox_path: Option<String>,              // custom Camoufox binary path
  }
  ```
- All new fields use `Option<T>` with `#[serde(default)]` so existing settings.json files stay compatible

**Frontend — New component (`src/lib/components/ui/SettingsPage.svelte`):**
- Render as a full-screen modal (like InstanceSettingsModal pattern)
- Sections (collapsible, reusing `SettingsSection.svelte`):
  1. **General** — default view mode (grid/list toggle), default sort (field + direction)
  2. **Confirmations** — skip wipe confirmation, skip delete confirmation (moved from inline)
  3. **Default Templates** — default proxy config (reuse ProxySection), default fingerprint config
  4. **Paths** — Camoufox binary path (with browse button using Tauri dialog)
  5. **Data** — Export all data / Import data (Phase 2 integration point)
  6. **About** — App version, Camoufox version, links

**Frontend (`src/routes/+layout.svelte`):**
- Add settings icon button in header bar
- Add `showSettingsPage` state
- Render `<SettingsPage show={showSettingsPage} on:close />`

**Frontend (`src/lib/store.ts`):**
- Update `AppSettings` interface with new fields
- Update `loadSettings`/`updateSettings` to handle new fields

**Frontend (`src/routes/+page.svelte`):**
- On mount: apply `default_view_mode`, `default_sort_field`, `default_sort_dir` from settings (instead of localStorage, migrate to settings)

---

## Phase 2: Instance Management Features

### 2.1 Instance Tags (Color-coded)

**Data model:** Each instance gets an optional `tags` array. Tags are simple `{ label: string, color: string }` objects. A global tag registry is maintained in settings.

**Backend (`src-tauri/src/instances.rs`):**
- Add to `InstanceConfig`:
  ```rust
  pub tags: Option<Vec<String>>,  // tag labels, #[serde(default)]
  ```
- Add new command `update_instance_tags(app: AppHandle, id: String, tags: Vec<String>) -> Result<(), String>`
  - Updates `anon_config.json` with new tags array

**Backend (`src-tauri/src/settings.rs`):**
- Add to `AppSettings`:
  ```rust
  pub tag_definitions: Option<Vec<TagDefinition>>,  // #[serde(default)]
  ```
  ```rust
  pub struct TagDefinition {
      pub label: String,
      pub color: String,  // hex color
  }
  ```
- Add commands: `create_tag`, `delete_tag`, `update_tag`

**Frontend (`src/lib/store.ts`):**
- Add `TagDefinition` interface
- Update `AppSettings` and `InstanceConfig` interfaces
- Add `updateInstanceTags(id, tags)`, `createTag(label, color)`, `deleteTag(label)`, `updateTag(oldLabel, newLabel, color)`

**Frontend (`src/lib/components/instance/InstanceCard.svelte`):**
- Render colored tag pills below instance name (both grid and list views)
- Click tag to filter by that tag
- Settings modal: add tag selector section with multi-select dropdown

**Frontend — New component (`src/lib/components/ui/TagManager.svelte`):**
- Create/edit/delete tags (inline in settings page)
- Color picker (predefined palette: 8 colors)
- Used in both SettingsPage (global management) and InstanceSettingsModal (assign to instance)

**Frontend (`src/routes/+page.svelte`):**
- Populate tag filter dropdown from `$settings.tag_definitions`
- Filter by tag in `filteredInstances`

### 2.2 Instance Notes/Description

**Backend (`src-tauri/src/instances.rs`):**
- Add to `InstanceConfig`:
  ```rust
  pub notes: Option<String>,  // #[serde(default)]
  ```
- Add command `update_instance_notes(app: AppHandle, id: String, notes: Option<String>) -> Result<(), String>`

**Frontend (`src/lib/store.ts`):**
- Update `InstanceConfig` interface
- Add `updateInstanceNotes(id, notes)`

**Frontend (`src/lib/components/instance/InstanceCard.svelte`):**
- Grid view: show truncated notes below tags (if present), with tooltip for full text
- List view: show notes icon indicator if notes exist, tooltip shows content

**Frontend (`src/lib/components/instance/InstanceSettingsModal.svelte`):**
- Add "NOTES" textarea at top of settings modal (above tabs), always visible
- Auto-save on close or explicit save

### 2.3 Bulk Operations (Multi-select)

**Frontend (`src/routes/+page.svelte`):**
- Add state: `selectedInstances: Set<string>`, `selectMode = false`
- Add "SELECT" toggle button in header (next to view mode toggle)
- When select mode is on:
  - Show checkboxes on each InstanceCard
  - Show bulk action bar at bottom: "X SELECTED" + "LAUNCH ALL" + "STOP ALL" + "DELETE ALL" + "DESELECT"
  - "Select All" / "Deselect All" shortcuts
- Keyboard: Ctrl+A to select all (when in select mode)

**Frontend (`src/lib/components/instance/InstanceCard.svelte`):**
- New props: `selectable = false`, `selected = false`
- New event: `on:select` (dispatches with instance id)
- When `selectable`: show checkbox before status dot, clicking card toggles selection

**Frontend (`src/lib/store.ts`):**
- Add `bulkLaunch(ids: string[]): Promise<void>` — sequential launch with delay
- Add `bulkStop(ids: string[]): Promise<void>` — parallel stop
- Add `bulkDelete(ids: string[]): Promise<void>` — sequential delete with single confirmation

### 2.4 Import/Export

**Backend (`src-tauri/src/instances.rs`):**
- Add command `export_instance(app: AppHandle, id: String) -> Result<String, String>`
  - Returns JSON string of the instance config (fingerprint + proxy, no browser data)
- Add command `export_all_instances(app: AppHandle) -> Result<String, String>`
  - Returns JSON string with `{ version: "1.0", instances: [...], settings: {...}, tags: [...] }`
- Add command `import_instances(app: AppHandle, json: String) -> Result<Vec<InstanceConfig>, String>`
  - Parses JSON, creates new instances with new UUIDs, handles name conflicts (append " (imported)")
  - Returns list of created instances

**Frontend (`src/lib/store.ts`):**
- Add `exportInstance(id)`, `exportAllInstances()`, `importInstances(json)`

**Frontend — Integration:**
- InstanceCard: add "EXPORT" option in a context menu or settings modal
- SettingsPage: "EXPORT ALL" and "IMPORT" buttons in Data section
- Use Tauri `dialog.save()` / `dialog.open()` for file picker
- Toast notifications for success/error

---

## Phase 3: Proxy & Fingerprint Features

### 3.1 Proxy Testing (Full Diagnostic)

**Backend — New module (`src-tauri/src/proxy_tester.rs`):**
- Add command `test_proxy(proxy_config: ProxyConfig) -> Result<ProxyTestResult, String>`
- `ProxyTestResult` struct:
  ```rust
  pub struct ProxyTestResult {
      pub success: bool,
      pub ip: Option<String>,
      pub country: Option<String>,
      pub city: Option<String>,
      pub latency_ms: Option<u64>,
      pub dns_leak: Option<bool>,      // true if DNS leaks detected
      pub error: Option<String>,
  }
  ```
- Implementation:
  1. Build `reqwest::Client` with proxy configured from `ProxyConfig`
  2. Measure time, request `https://ipapi.co/json/` (or similar free API) through proxy
  3. Parse response for IP, country, city
  4. DNS leak: make a secondary request to `https://dnsleaktest.com/api` or check if resolved IP matches proxy IP
  5. Return `ProxyTestResult`

**Backend (`src-tauri/src/lib.rs`):**
- Register `test_proxy` command
- Add `mod proxy_tester;`

**Frontend (`src/lib/store.ts`):**
- Add `ProxyTestResult` interface
- Add `testProxy(proxyConfig: ProxyConfig): Promise<ProxyTestResult>`

**Frontend (`src/lib/components/instance/settings/ProxySection.svelte`):**
- Add "TEST PROXY" button (disabled when no proxy configured)
- On click: show loading state, call `testProxy()`, show result panel:
  - Success: green panel with IP, location (country/city), latency, DNS leak status
  - Failure: red panel with error message
- Result panel dismissible, shows below proxy config form

### 3.2 Proxy Rotation

**Backend (`src-tauri/src/instances.rs`):**
- Add to `InstanceConfig`:
  ```rust
  pub proxy_pool: Option<Vec<ProxyConfig>>,         // #[serde(default)]
  pub proxy_rotation_mode: Option<String>,           // "sequential" | "random", #[serde(default)]
  pub proxy_rotation_index: Option<usize>,           // current index for sequential, #[serde(default)]
  ```
- Modify `launch_instance()`:
  - If `proxy_pool` is non-empty: select next proxy based on `proxy_rotation_mode`
  - If sequential: use `proxy_rotation_index % pool.len()`, increment and save
  - If random: pick random index
  - Apply selected proxy to `user.js` for this launch (not modifying `proxy_config`)
  - Store which proxy was used in the launch log (for display)

**Frontend (`src/lib/store.ts`):**
- Update `InstanceConfig` interface with new fields
- Add `updateProxyPool(id, pool, mode)`

**Frontend — New component (`src/lib/components/instance/settings/ProxyPoolSection.svelte`):**
- Render below ProxySection in the Proxy tab
- "PROXY ROTATION" collapsible section
- Mode selector: OFF / SEQUENTIAL / RANDOM
- When ON: editable list of proxies
  - Each row: type select, host, port, username, password, remove button
  - "ADD PROXY" button at bottom
  - Reuse ProxyConfig inputs (extract shared proxy form sub-component)
  - Drag-to-reorder (or up/down buttons) for sequential mode
- Show "CURRENT INDEX: X of Y" for sequential mode

### 3.3 Fingerprint Rotation

**Backend (`src-tauri/src/instances.rs`):**
- Add to `InstanceConfig`:
  ```rust
  pub fingerprint_pool: Option<Vec<FingerprintConfig>>,  // #[serde(default)]
  pub fingerprint_rotation_mode: Option<String>,          // "sequential" | "random", #[serde(default)]
  pub fingerprint_rotation_index: Option<usize>,          // #[serde(default)]
  ```
- Modify `launch_instance()`:
  - If `fingerprint_pool` is non-empty: select next fingerprint based on rotation mode
  - Apply selected fingerprint's CAMOU_CONFIG for this launch
  - Increment index for sequential

**Frontend (`src/lib/store.ts`):**
- Update `InstanceConfig` interface
- Add `updateFingerprintPool(id, pool, mode)`

**Frontend — New section in InstanceSettingsModal:**
- Add "FINGERPRINT ROTATION" collapsible panel above the fingerprint sections (below AUTO MODE)
- Mode selector: OFF / SEQUENTIAL / RANDOM
- When ON: show numbered list of saved fingerprint profiles
  - Each entry: summary line (UA snippet, screen res, WebGL vendor)
  - "SAVE CURRENT AS PROFILE" button — snapshots current fp fields as a new pool entry
  - Remove button per entry
  - Edit button per entry — loads that profile into the main fp editor
- Note: if rotation is ON, the main fingerprint fields show "EDITING POOL ENTRY X" or "BASE CONFIG (used when pool is empty)"

### 3.4 Fingerprint Test Sites

**Backend (`src-tauri/src/instances.rs`):**
- Modify `launch_instance()` to accept an optional `startup_url: Option<String>` parameter
- Pass URL as additional argument to Camoufox: `--url <startup_url>` or via `CAMOU_CONFIG` if supported
- If Camoufox doesn't support startup URLs natively, use environment variable or command-line arg

**Frontend (`src/lib/store.ts`):**
- Modify `launchInstance(id, startupUrl?)` to pass optional URL

**Frontend (`src/lib/components/instance/InstanceCard.svelte`):**
- Add a dropdown/chevron next to the LAUNCH button
- Dropdown options:
  - "LAUNCH" (default, no URL)
  - "LAUNCH → BROWSERLEAKS.COM"
  - "LAUNCH → CREEPJS.COM"  
  - "LAUNCH → IPHEY.COM"
  - "LAUNCH → PIXELSCAN.NET"
- Clicking an option launches with the corresponding URL

---

## Phase 4: Data Management

### 4.1 Session/Cookie Viewer (View & Clear All)

**Backend — New module (`src-tauri/src/session_manager.rs`):**
- Add command `get_session_info(app: AppHandle, id: String) -> Result<SessionInfo, String>`
  - `SessionInfo` struct:
    ```rust
    pub struct SessionInfo {
        pub profile_size_bytes: u64,        // total profile directory size
        pub cookies_count: Option<u32>,      // parse cookies.sqlite if exists
        pub local_storage_size: Option<u64>, // webappsstore.sqlite size
        pub cache_size: Option<u64>,         // cache2/ directory size
        pub session_store_exists: bool,      // sessionstore.jsonlz4 exists
        pub history_count: Option<u32>,      // places.sqlite moz_places count
    }
    ```
  - Implementation: walk the instance profile directory, read SQLite databases with `rusqlite` (add as dependency), calculate sizes
- Add command `clear_session_data(app: AppHandle, id: String, types: Vec<String>) -> Result<(), String>`
  - `types`: subset of `["cookies", "localStorage", "cache", "sessions", "history", "all"]`
  - Deletes corresponding files/directories from the profile
  - Instance must NOT be running

**Backend (`src-tauri/src/lib.rs`):**
- Register both commands
- Add `mod session_manager;`

**Backend (`Cargo.toml`):**
- Add `rusqlite` dependency with `bundled` feature

**Frontend (`src/lib/store.ts`):**
- Add `SessionInfo` interface
- Add `getSessionInfo(id)`, `clearSessionData(id, types)`

**Frontend — New component (`src/lib/components/instance/SessionPanel.svelte`):**
- Accessible from InstanceCard (new "DATA" button or tab in settings modal)
- Shows:
  - Profile size (formatted: KB/MB/GB)
  - Cookie count
  - LocalStorage size
  - Cache size
  - History entry count
  - Session store status
- "CLEAR ALL" button (with confirmation) — calls `clearSessionData(id, ['all'])`
- Individual clear buttons per data type
- Disabled/hidden when instance is running (show message: "Stop instance to manage data")

---

## File Change Summary

### New Files
| File | Description |
|---|---|
| `src-tauri/src/proxy_tester.rs` | Proxy testing logic |
| `src-tauri/src/session_manager.rs` | Session/cookie data inspection and clearing |
| `src/lib/components/ui/Toast.svelte` | Toast notification component |
| `src/lib/components/ui/SettingsPage.svelte` | Global settings modal |
| `src/lib/components/ui/TagManager.svelte` | Tag CRUD component |
| `src/lib/components/instance/settings/ProxyPoolSection.svelte` | Proxy rotation pool editor |
| `src/lib/components/instance/SessionPanel.svelte` | Session data viewer/clearer |

### Modified Files
| File | Changes |
|---|---|
| `src-tauri/src/lib.rs` | New module declarations, new commands in handler |
| `src-tauri/src/instances.rs` | New fields (tags, notes, proxy_pool, fingerprint_pool, rotation), rename/export/import commands, launch modifications |
| `src-tauri/src/settings.rs` | Expanded AppSettings, TagDefinition struct |
| `src-tauri/Cargo.toml` | Add `rusqlite` dependency |
| `src/lib/store.ts` | New interfaces, stores, functions for all features |
| `src/routes/+layout.svelte` | Settings icon, Toast component, global settings modal |
| `src/routes/+page.svelte` | Filter bar, bulk selection, search state |
| `src/lib/components/instance/InstanceCard.svelte` | Inline rename, tags, notes, selectable, launch dropdown |
| `src/lib/components/instance/InstanceSettingsModal.svelte` | Notes textarea, fingerprint rotation section, tag selector |
| `src/lib/components/instance/settings/ProxySection.svelte` | Test proxy button + result panel |
| `src/app.css` | Toast styles, filter bar styles, tag pill styles |

### Implementation Order
1. Phase 1.3 — Toast system (needed by everything else)
2. Phase 1.1 — Instance renaming
3. Phase 1.2 — Search & filter bar
4. Phase 1.4 — Settings page
5. Phase 2.1 — Tags
6. Phase 2.2 — Notes
7. Phase 2.3 — Bulk operations
8. Phase 2.4 — Import/export
9. Phase 3.1 — Proxy testing
10. Phase 3.4 — Fingerprint test sites
11. Phase 3.2 — Proxy rotation
12. Phase 3.3 — Fingerprint rotation
13. Phase 4.1 — Session viewer
