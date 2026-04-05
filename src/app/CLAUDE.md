# App Module

Application state machine and event handling.

## Architecture: Action-based Message Passing

```
EventHandler (tokio task)
    ↓ Action enum
App.update(action)
    ↓ &AppState
ui::draw(frame, &app_state)
```

## Components

### `actions.rs` — Action Enum
- All user and system actions in a single enum
- EventHandler converts crossterm events into Actions
- App.update() processes Actions and updates state
- Actions: Tick, Render, Quit, NextTab, PrevTab, ScrollUp, ScrollDown, ScrollLeft, ScrollRight, NextFile, PrevFile, ToggleHelp, CycleSortColumn, ToggleSortDirection, EnterSearchMode, ExitSearchMode, ConfirmSearch, SearchInput(char), SearchBackspace, Resize, SplashStatus, LoadComplete, Error

### `state.rs` — AppState
- `active_tab: ActiveTab` — which tab is active (default: Summary)
- `should_quit: bool` — quit flag
- `show_help: bool` — help overlay
- `loading: bool` — true while files are loading (splash screen is shown)
- `error_message: Option<String>` — error message
- `qc_results: Option<QcResults>` — parsed data
- Per-tab selection indices: `samtools_selected`, `bcftools_selected`, `fastqc_selected`
- `scroll_offset: u16`
- `sort_column: SortColumn` — active sort column in the Overview table (File/Tool/Summary/Status)
- `sort_ascending: bool` — sort direction
- `search_active: bool` — whether search mode is active
- `search_input: String` — current search text (while typing)
- `search_confirmed: String` — confirmed filter (after Enter)
- `search_active_flag: Arc<AtomicBool>` — search state shared with EventHandler
- `splash_tick: u16` — splash animation tick counter (incremented every 200ms)
- `splash_done: bool` — whether splash has completed
- `splash_status: String` — status message displayed on splash screen
- `pending_results: Option<QcResults>` — data received before splash finishes is buffered here

### `state.rs` — SortColumn Enum
- `File` → `Tool` → `Summary` → `Status` → `File` (cycle, for Overview tab)
- Cyclic transition via `next()` method

### `state.rs` — SummarySortColumn Enum
- 17 variants: File, Tool, Reads, MappedPct, DupPct, ErrorRate, AvgQuality, Variants, Snps, Indels, TsTv, TotalSeqs, GcPct, PassModules, WarnModules, FailModules, OverallQc
- `next()` and `index()` methods
- Cycled with `s` key in the Summary tab

### `state.rs` — ActiveTab Enum
- `Summary` — MultiQC-style wide table (default opening tab)
- `Overview` — aggregate dashboard
- `Samtools` — samtools stats detail view
- `Bcftools` — bcftools stats detail view
- `Fastqc` — FastQC detail view

### `mod.rs` — App update logic
- State updates via `update(action: Action)`
- NextFile/PrevFile updates the relevant selection index based on active tab
- NextFile/PrevFile is a no-op on the Overview tab
- Search actions use `set_search_active()` to update both local state and `Arc<AtomicBool>`
- CycleSortColumn is context-aware: `summary_sort_column.next()` in Summary tab, `sort_column.next()` in others
- ScrollLeft/ScrollRight only updates `summary_horizontal_offset` in the Summary tab
- NextFile/PrevFile updates `summary_selected` in the Summary tab
- `thresholds: ThresholdConfig` — QC threshold rules (loadable from TOML or default)
- `Action::SplashStatus` handler: updates `splash_status` message
- `Action::Render` handler: increments `splash_tick` during loading; transitions `pending_results` to `qc_results` and sets `loading=false` after 16 ticks + data ready
- `Action::LoadComplete`: buffers data to `pending_results` if splash hasn't finished; otherwise sets `qc_results` directly

## Rules

- AppState must not derive `Clone` or `Copy` (may contain large data)
- State mutation must only happen inside `update()`
- UI render functions must not modify state (`&self` or `&AppState`)
- Keyboard events must still be processed during loading (quit, etc.)
- Terminal restore must be guaranteed via panic hook
