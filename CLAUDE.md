# QCForge

Biyoinformatik QC dashboard TUI uygulaması. FastQC, samtools stats ve bcftools stats çıktılarını parse edip terminal üzerinde interaktif görselleştirme sunar.

## Tech Stack

- **Dil:** Rust (edition 2021)
- **TUI:** ratatui 0.30 + crossterm 0.29 (event-stream feature)
- **Async:** tokio (full features)
- **Error handling:** thiserror (custom error types, anyhow kullanılmaz)
- **CLI:** clap 4.5 (derive feature)
- **FastQC zip:** zip crate
- **Serialization:** serde + serde_json (JSON export için)
- **File discovery:** glob

## Komutlar

```bash
cargo build              # Derleme
cargo run -- <DIR>       # TUI başlat (dizin tarayarak)
cargo run -- .           # Mevcut dizini tara
cargo test               # Tüm testleri çalıştır
cargo clippy             # Lint kontrolü
cargo fmt                # Kod formatlama
```

## Mimari

```
src/
├── main.rs       # Entry point, terminal setup/restore, tokio runtime
├── cli.rs        # clap derive ile CLI argümanları
├── error.rs      # QcForgeError enum (thiserror)
├── app/          # Uygulama state machine (Action pattern)
├── event/        # crossterm event handling (async EventStream)
├── parser/       # Dosya parser'ları (samtools, bcftools, fastqc)
├── scanner/      # Dizin tarama ve dosya tipi tespiti
└── ui/           # ratatui render katmanı (tabs + widgets)
```

## Kod Kuralları

- **Error handling:** Tüm hatalar `QcForgeError` enum'u üzerinden. `unwrap()` veya `expect()` sadece test kodunda kullanılabilir. Production kodda `?` operatörü ile propagation.
- **Naming:** Rust standart snake_case. Struct isimleri PascalCase. Module isimleri snake_case.
- **Imports:** `use crate::` ile internal imports. Wildcard import (`use x::*`) yasak, açık import kullanılmalı.
- **Clippy:** `cargo clippy` warning'siz geçmeli. `#[allow(...)]` sadece gerekçe yorumuyla.
- **Tests:** Her parser modülü kendi unit test'lerini içermeli (`#[cfg(test)] mod tests`). Test verileri `include_str!` veya inline string ile.
- **Async:** Dosya I/O tokio::spawn ile arka planda. TUI event loop hiçbir zaman bloklanmamalı.
- **Terminal restore:** Panic durumunda bile terminal state restore edilmeli (panic hook kullan).

## Dikkat Edilecekler

- samtools stats ve bcftools stats formatları benzer ama farklı. SN section'ındaki field index'leri farklıdır.
- FastQC zip arşivleri içinde `*/fastqc_data.txt` yolu değişkenlik gösterebilir, glob pattern ile aranmalı.
- ratatui her frame'de tüm UI'ı yeniden çizer (immediate mode). State'i UI'dan ayır.
- crossterm event-stream feature'ı futures StreamExt gerektirir.
