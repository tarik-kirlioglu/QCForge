# App Module

Uygulama state machine ve event handling.

## Mimari: Action-based Message Passing

```
EventHandler (tokio task)
    ↓ Action enum
App.update(action)
    ↓ &AppState
ui::draw(frame, &app_state)
```

## Bileşenler

### `actions.rs` — Action Enum
- Tüm kullanıcı ve sistem eylemleri tek bir enum'da
- EventHandler crossterm event'lerini Action'a dönüştürür
- App.update() Action'ları işler ve state'i günceller
- Action'lar: Tick, Render, Quit, NextTab, PrevTab, ScrollUp, ScrollDown, ScrollLeft, ScrollRight, NextFile, PrevFile, ToggleHelp, CycleSortColumn, ToggleSortDirection, EnterSearchMode, ExitSearchMode, ConfirmSearch, SearchInput(char), SearchBackspace, Resize, LoadComplete, Error

### `state.rs` — AppState
- `active_tab: ActiveTab` — hangi tab aktif (default: Overview)
- `should_quit: bool` — çıkış flag'i
- `show_help: bool` — help overlay
- `loading: bool` — dosyalar yüklenirken true (splash screen gösterilir)
- `error_message: Option<String>` — hata mesajı
- `qc_results: Option<QcResults>` — parse edilmiş veriler
- Per-tab selection index'leri: `samtools_selected`, `bcftools_selected`, `fastqc_selected`
- `scroll_offset: u16`
- `sort_column: SortColumn` — Overview tablosunda aktif sıralama sütunu (File/Tool/Summary/Status)
- `sort_ascending: bool` — sıralama yönü
- `search_active: bool` — arama modu açık mı
- `search_input: String` — anlık arama metni (yazarken)
- `search_confirmed: String` — onaylanmış filtre (Enter sonrası)
- `search_active_flag: Arc<AtomicBool>` — EventHandler ile paylaşılan search state
- `splash_tick: u16` — splash animasyon tick sayacı (250ms aralıkla artırılır)
- `splash_done: bool` — splash tamamlandı mı
- `pending_results: Option<QcResults>` — splash bitmeden gelen veriler burada buffer'lanır

### `state.rs` — SortColumn Enum
- `File` → `Tool` → `Summary` → `Status` → `File` (cycle, Overview tab için)
- `next()` metodu ile döngüsel geçiş

### `state.rs` — SummarySortColumn Enum
- 17 variant: File, Tool, Reads, MappedPct, DupPct, ErrorRate, AvgQuality, Variants, Snps, Indels, TsTv, TotalSeqs, GcPct, PassModules, WarnModules, FailModules, OverallQc
- `next()` ve `index()` metodları
- Summary tab'da `s` tuşu ile cycle edilir

### `state.rs` — ActiveTab Enum
- `Overview` — aggregate dashboard (default açılış tab'ı)
- `Samtools` — samtools stats detay görünümü
- `Bcftools` — bcftools stats detay görünümü
- `Fastqc` — FastQC detay görünümü

### `mod.rs` — App update logic
- `update(action: Action)` ile state güncellemesi
- NextFile/PrevFile aktif tab'a göre ilgili selection index'i günceller
- Overview tab'da NextFile/PrevFile no-op
- Search action'ları `set_search_active()` ile hem local state hem `Arc<AtomicBool>` günceller
- CycleSortColumn context-aware: Summary tab'da `summary_sort_column.next()`, diğerlerinde `sort_column.next()`
- ScrollLeft/ScrollRight sadece Summary tab'da `summary_horizontal_offset` günceller
- NextFile/PrevFile Summary tab'da `summary_selected` günceller
- `thresholds: ThresholdConfig` — QC eşik kuralları (TOML'dan yüklenebilir veya default)
- `Action::Render` handler'ı: loading sırasında `splash_tick` artırır, 8 tick + veri hazır olunca `pending_results`'ı `qc_results`'a taşıyıp `loading=false` yapar
- `Action::LoadComplete`: splash bitmemişse veriyi `pending_results`'a buffer'lar, bitmişse direkt `qc_results`'a atar

## Kurallar

- App state'i `Clone` veya `Copy` derive etmemeli (büyük veri içerebilir)
- State mutation sadece `update()` içinde olmalı
- UI render fonksiyonları state'i değiştirmemeli (`&self` veya `&AppState`)
- Loading sırasında keyboard event'leri yine de işlenmeli (quit vb.)
- Panic hook ile terminal restore garantilenmeli
