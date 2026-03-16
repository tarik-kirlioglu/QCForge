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
- Action'lar: Tick, Render, Quit, NextTab, PrevTab, ScrollUp, ScrollDown, NextFile, PrevFile, ToggleHelp, Resize, LoadComplete, Error

### `state.rs` — AppState
- `active_tab: ActiveTab` — hangi tab aktif (default: Overview)
- `should_quit: bool` — çıkış flag'i
- `show_help: bool` — help overlay
- `loading: bool` — dosyalar yüklenirken true
- `error_message: Option<String>` — hata mesajı
- `qc_results: Option<QcResults>` — parse edilmiş veriler
- Per-tab selection index'leri: `samtools_selected`, `bcftools_selected`, `fastqc_selected`
- `scroll_offset: u16`

### `state.rs` — ActiveTab Enum
- `Overview` — aggregate dashboard (default açılış tab'ı)
- `Samtools` — samtools stats detay görünümü
- `Bcftools` — bcftools stats detay görünümü
- `Fastqc` — FastQC detay görünümü

### `mod.rs` — App update logic
- `update(action: Action)` ile state güncellemesi
- NextFile/PrevFile aktif tab'a göre ilgili selection index'i günceller
- Overview tab'da NextFile/PrevFile no-op

## Kurallar

- App state'i `Clone` veya `Copy` derive etmemeli (büyük veri içerebilir)
- State mutation sadece `update()` içinde olmalı
- UI render fonksiyonları state'i değiştirmemeli (`&self` veya `&AppState`)
- Loading sırasında keyboard event'leri yine de işlenmeli (quit vb.)
- Panic hook ile terminal restore garantilenmeli
