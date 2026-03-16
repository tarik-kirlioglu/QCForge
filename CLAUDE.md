# QCForge

Biyoinformatik QC dashboard TUI uygulaması. FastQC, samtools stats ve bcftools stats çıktılarını parse edip terminal üzerinde interaktif görselleştirme sunar. BAM/VCF dosyalarından otomatik stats üretme desteği vardır.

## Tech Stack

- **Dil:** Rust (edition 2021)
- **TUI:** ratatui 0.30 + crossterm 0.29 (event-stream feature)
- **Async:** tokio (full features)
- **Error handling:** thiserror (custom error types, anyhow kullanılmaz)
- **CLI:** clap 4.5 (derive feature)
- **FastQC zip:** zip 8.2
- **Serialization:** serde + serde_json (JSON export için)
- **File discovery:** glob
- **Stream:** futures (crossterm EventStream için)

## Komutlar

```bash
cargo build                          # Derleme
cargo run -- <DIR>                   # TUI başlat (dizin tarayarak)
cargo run -- .                       # Mevcut dizini tara
cargo run -- -g <DIR>                # BAM/VCF'ten stats üret + TUI
cargo run -- --generate <DIR>        # Uzun form
cargo run -- -g --output-dir ./out/ <DIR>  # Stats'ları farklı dizine yaz
cargo run -- --export-json qc.json <DIR>   # JSON export (TUI'sız)
cargo run -- -g --export-json qc.json <DIR> # Generate + JSON export
cargo test                           # Tüm testleri çalıştır (18 test)
cargo clippy                         # Lint kontrolü
cargo fmt                            # Kod formatlama
```

## Mimari

```
src/
├── main.rs        # Entry point, terminal setup/restore, tokio runtime
├── cli.rs         # clap derive ile CLI argümanları
├── error.rs       # QcForgeError enum (thiserror)
├── app/           # Uygulama state machine (Action pattern)
├── event/         # crossterm event handling (async EventStream)
├── generator/     # BAM/VCF → stats dosyası üretimi (samtools/bcftools çalıştırma)
├── parser/        # Dosya parser'ları (samtools, bcftools, fastqc)
├── scanner/       # Dizin tarama, dosya tipi tespiti (stats + BAM/VCF)
└── ui/            # ratatui render katmanı (4 tab + widgets)
    ├── tabs/      # Overview, samtools, bcftools, FastQC
    └── widgets/   # gauge, table helpers
```

## CLI Flags

| Flag | Kısa | Açıklama |
|------|------|----------|
| `<DIR>` | | Taranacak dizin (default: `.`) |
| `--generate` | `-g` | BAM/VCF bulunca otomatik samtools/bcftools stats çalıştır |
| `--output-dir` | | Generate edilen stats dosyalarının yazılacağı dizin |
| `--export-json` | | JSON export (TUI açılmaz) |
| `--filter` | `-f` | Sadece belirli araç göster (samtools/bcftools/fastqc) |
| `--max-depth` | | Recursive tarama derinliği (default: 5) |

## Kod Kuralları

- **Error handling:** Tüm hatalar `QcForgeError` enum'u üzerinden. `unwrap()` veya `expect()` sadece test kodunda kullanılabilir. Production kodda `?` operatörü ile propagation.
- **Naming:** Rust standart snake_case. Struct isimleri PascalCase. Module isimleri snake_case.
- **Imports:** `use crate::` ile internal imports. Wildcard import (`use x::*`) yasak, açık import kullanılmalı.
- **Clippy:** `cargo clippy` warning'siz geçmeli. `#[allow(...)]` sadece gerekçe yorumuyla.
- **Tests:** Her parser modülü kendi unit test'lerini içermeli (`#[cfg(test)] mod tests`). Test verileri inline string ile. Şu an 18 test mevcut.
- **Async:** Dosya I/O tokio::spawn ile arka planda. FastQC zip işlemi spawn_blocking ile. TUI event loop hiçbir zaman bloklanmamalı.
- **Terminal restore:** Panic durumunda bile terminal state restore edilmeli (panic hook kullan).

## Dikkat Edilecekler

- samtools stats ve bcftools stats formatları benzer ama farklı. SN section'ındaki field index'leri farklıdır.
- bcftools stats DP section'ında bin değerleri string olabilir (`>500` gibi). `DpEntry.bin` tipi `String`.
- FastQC zip arşivleri içinde `*/fastqc_data.txt` yolu değişkenlik gösterebilir, `ends_with` ile aranır.
- `##FastQC` header satırı `>>` ile başlamaz, `#` ile başlar — section parser'da dikkat.
- ratatui her frame'de tüm UI'ı yeniden çizer (immediate mode). State'i UI'dan ayır.
- crossterm event-stream feature'ı futures StreamExt gerektirir.
- `--generate` modu stats dosyası zaten varsa skip eder (idempotent).
- Generator, samtools/bcftools'un PATH'de olup olmadığını kontrol eder.
