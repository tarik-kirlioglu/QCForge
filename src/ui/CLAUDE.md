# UI Module

Terminal render layer using ratatui. 5 tabs, splash screen, help overlay, loading/error states.

## Structure

```
ui/
├── mod.rs        # Root render fn: draw(frame, &app_state) + splash screen + loading/error/help overlay
├── layout.rs     # Header, tab bar, content area, footer layout sections
├── tabs/
│   ├── overview.rs   # Aggregate QC dashboard (file list, aggregate stats, gauges)
│   ├── samtools.rs   # samtools stats detail: summary table + mapping/dup/paired gauges
│   ├── bcftools.rs   # bcftools stats detail: summary + Ts/Tv + substitution types + indel distribution
│   └── fastqc.rs     # FastQC detail: basic stats + module status (pass/warn/fail) + per-base quality chart
└── widgets/
    ├── gauge.rs  # Quality color calculation (mapping_style, duplication_style)
    └── table.rs  # Styled table helpers (header_style, highlight_style)
```

## Tabs

| Tab | Content |
|-----|---------|
| Summary | MultiQC-style wide table: all samples + all metrics, threshold-colored cells, h/l horizontal scroll |
| Overview | File counts, aggregate stats, avg mapping/dup gauge, sortable + filterable file list |
| samtools | Summary Numbers table + Mapping/Duplication/Properly Paired gauges |
| bcftools | Summary + Ts/Tv + Substitution Types (inline bar, Ts=cyan Tv=magenta) + InDel Distribution (del=red ins=green) |
| FastQC | Basic Statistics + Module Status (colored PASS/WARN/FAIL) + Per Base Quality (bar chart) |

## Color Scheme

| Status | Color | Usage |
|--------|-------|-------|
| PASS / Good | Green | mapping > 90%, Q >= 28, dup <= 10% |
| WARN / Marginal | Yellow | mapping 80-90%, Q 20-28, dup 10-20% |
| FAIL / Bad | Red | mapping < 80%, Q < 20, dup > 20% |
| Header/Border | Cyan | Frames and headings |
| Transitions | Cyan | bcftools substitution types (A>G, G>A, C>T, T>C) |
| Transversions | Magenta | bcftools substitution types (others) |
| Insertions | Green | bcftools indel dist (positive length) |
| Deletions | Red | bcftools indel dist (negative length) |
| Normal text | White | Data display |
| Secondary | Gray / DarkGray | Comments, descriptions, secondary info |
| Splash Logo | Rgb(180,220,255) | QCForge ASCII logo (ice-blue, bold, fade-in) |
| DNA Base A | Rgb(80,220,100) | Green — adenine on helix strands |
| DNA Base T | Rgb(220,60,60) | Red — thymine on helix strands |
| DNA Base G | Rgb(60,140,255) | Blue — guanine on helix strands |
| DNA Base C | Rgb(255,180,40) | Amber — cytosine on helix strands |
| DNA Cross-links | Rgb(200,170,80) | Golden — hydrogen bonds between strands |
| Helix Glow | Rgb(30,50,70) | Dark navy — faint glow near helix |
| Background dots | Rgb(50,40,70) | Dark purple — sparse scattered dots |
| Splash Loading | Cyan | Dynamic status text with animated dots |

## Splash Screen

- Rendered via `render_splash(frame, tick, status)` function
- ASCII QCForge logo appears with a fade-in effect (3 more characters revealed per tick)
- Subtitle: "Terminal QC Dashboard for Bioinformatics"
- Dynamic status text from `splash_status` with animated dots (cycles 0-3 dots every 3 ticks)
- Background: animated double-helix wave pattern (two sine waves with pi offset) using colored DNA bases (A/T/G/C), golden cross-links between strands, faint glow near helix, sparse dark dots in background
- Minimum terminal size: 10x10 (renders empty below that)

## Layout Rules

- ratatui immediate mode: entire UI is redrawn every frame
- State is passed to UI functions as `&AppState` reference; UI functions do not modify state
- Layout uses `ratatui::layout::Layout` with constraint-based sizing
- Active tab is highlighted in tab bar (underlined + bold + cyan)
- Footer shows keybinding info; when search mode is active, footer becomes a search bar
- Overview tab builds file list via `OverviewRow` struct; order is sort → filter → render
- Active sort column shows `▲`/`▼` indicator in the header
- Active filter is shown in footer as `[filter: xxx]`
- Each tab has a file header: `tool: filename [n/total] n:Next p:Prev`
- Help overlay toggles with `?`, positioned center via centered_rect

## Keybindings

- `q` / `Esc`: Quit (in search mode, Esc clears the filter)
- `←` / `→` / `Tab`: Switch tabs
- `j` / `k` / `↑` / `↓`: Scroll
- `n` / `p`: Navigate between files
- `s`: Cycle sort column (context-aware: Overview vs Summary)
- `S`: Toggle sort direction (asc/desc)
- `/`: Enter search mode (real-time filtering, Enter to confirm, Esc to clear)
- `h` / `l`: Scroll columns horizontally in the Summary tab
- `Ctrl+C`: Quit
- `?`: Toggle help overlay
