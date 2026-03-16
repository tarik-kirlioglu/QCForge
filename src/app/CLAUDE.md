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
- Action'lar: Tick, Render, Quit, NextTab, PrevTab, ScrollUp, ScrollDown, NextFile, PrevFile, Select, ToggleHelp, Resize, LoadComplete, Error

### `state.rs` — AppState
- `active_tab: ActiveTab` — hangi tab aktif
- `should_quit: bool` — çıkış flag'i
- `show_help: bool` — help overlay
- `loading: bool` — dosyalar yüklenirken true
- `qc_results: Option<QcResults>` — parse edilmiş veriler
- Per-tab selection index'leri (overview_selected, samtools_selected, vb.)
- Scroll offset

### `mod.rs` — App struct
- `new()` ile başlangıç state'i
- `update(action: Action)` ile state güncellemesi
- Her Action match kolu basit ve side-effect free olmalı

## Kurallar

- App state'i `Clone` veya `Copy` derive etmemeli (büyük veri içerebilir)
- State mutation sadece `update()` içinde olmalı
- UI render fonksiyonları state'i değiştirmemeli (`&self` veya `&AppState`)
- Loading sırasında keyboard event'leri yine de işlenmeli (quit vb.)
- Panic hook ile terminal restore garantilenmeli
