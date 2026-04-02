# QCForge

Biyoinformatik QC dashboard TUI uygulaması. FastQC, samtools stats ve bcftools stats çıktılarını parse edip terminal üzerinde interaktif görselleştirme sunar. BAM/VCF dosyalarından otomatik stats üretme ve FASTQ dosyalarından FastQC çalıştırma desteği vardır.

## Tech Stack

- **Dil:** Rust (edition 2021)
- **TUI:** ratatui 0.30 + crossterm 0.29 (event-stream feature)
- **Async:** tokio (full features)
- **Error handling:** thiserror (custom error types, anyhow kullanılmaz)
- **CLI:** clap 4.5 (derive feature)
- **FastQC zip:** zip 8.2
- **Serialization:** serde + serde_json (JSON export için)
- **CSV export:** csv 1.3 (CSV/TSV export için)
- **Config:** toml 0.8 (threshold config dosyaları için)
- **File discovery:** glob
- **Stream:** futures (crossterm EventStream için)

## Komutlar

```bash
cargo build                          # Derleme
cargo run -- <DIR>                   # TUI başlat (dizin tarayarak)
cargo run -- .                       # Mevcut dizini tara
cargo run -- -g <DIR>                # BAM/VCF/FASTQ'dan stats üret + TUI
cargo run -- --generate <DIR>        # Uzun form
cargo run -- -g --output-dir ./out/ <DIR>  # Stats'ları farklı dizine yaz
cargo run -- --export-json qc.json <DIR>   # JSON export (TUI'sız)
cargo run -- --export-csv qc.csv <DIR>     # CSV export (TUI'sız)
cargo run -- --export-csv qc.tsv <DIR>     # TSV export (.tsv uzantısı tab-delimited)
cargo run -- --export-json qc.json --export-csv qc.csv <DIR>  # Her ikisi birden
cargo run -- -g --export-json qc.json <DIR> # Generate + JSON export
cargo run -- --thresholds custom.toml <DIR>  # Custom threshold config
cargo run -- --strict --export-csv out.csv <DIR>  # Strict mode (exit 1 on FAIL)
cargo test                           # Tüm testleri çalıştır (36 test)
cargo clippy                         # Lint kontrolü
cargo fmt                            # Kod formatlama
```

## Mimari

```
src/
├── main.rs        # Entry point, terminal setup/restore, tokio runtime
├── cli.rs         # clap derive ile CLI argümanları
├── error.rs       # QcForgeError enum (thiserror)
├── export.rs      # CSV/TSV export (QcRow flat struct, serde serialize, threshold-aware qc_status)
├── threshold.rs   # QC threshold engine (MetricThreshold, ThresholdConfig, TOML config)
├── app/           # Uygulama state machine (Action pattern)
├── event/         # crossterm event handling (async EventStream, Arc<AtomicBool> search state)
├── generator/     # BAM/VCF/FASTQ → stats dosyası üretimi (samtools/bcftools/fastqc çalıştırma)
├── parser/        # Dosya parser'ları (samtools, bcftools, fastqc)
├── scanner/       # Dizin tarama, dosya tipi tespiti (stats + BAM/VCF/FASTQ)
└── ui/            # ratatui render katmanı (5 tab + widgets)
    ├── tabs/      # Summary (threshold-colored), Overview (sortable + filterable), samtools, bcftools, FastQC
    └── widgets/   # gauge, table helpers
```

## CLI Flags

| Flag | Kısa | Açıklama |
|------|------|----------|
| `<DIR>` | | Taranacak dizin (default: `.`) |
| `--generate` | `-g` | BAM/VCF/FASTQ bulunca otomatik samtools/bcftools/fastqc çalıştır |
| `--output-dir` | | Generate edilen stats dosyalarının yazılacağı dizin |
| `--export-json` | | JSON export (TUI açılmaz) |
| `--export-csv` | | CSV/TSV export (TUI açılmaz, .tsv uzantısı tab-delimited) |
| `--thresholds` | | QC threshold config dosyası (TOML format, default: built-in eşikler) |
| `--strict` | | FAIL varsa exit code 1 ile çık (CI/CD entegrasyonu için) |
| `--filter` | `-f` | Sadece belirli araç göster (samtools/bcftools/fastqc) |
| `--max-depth` | | Recursive tarama derinliği (default: 5) |

## Kod Kuralları

- **Error handling:** Tüm hatalar `QcForgeError` enum'u üzerinden. `unwrap()` veya `expect()` sadece test kodunda kullanılabilir. Production kodda `?` operatörü ile propagation.
- **Naming:** Rust standart snake_case. Struct isimleri PascalCase. Module isimleri snake_case.
- **Imports:** `use crate::` ile internal imports. Wildcard import (`use x::*`) yasak, açık import kullanılmalı.
- **Clippy:** `cargo clippy` warning'siz geçmeli. `#[allow(...)]` sadece gerekçe yorumuyla.
- **Tests:** Her parser modülü kendi unit test'lerini içermeli (`#[cfg(test)] mod tests`). Test verileri inline string ile. Şu an 36 test mevcut (18 parser + 4 export + 11 threshold + 3 summary).
- **Async:** Dosya I/O tokio::spawn ile arka planda. FastQC zip işlemi spawn_blocking ile. TUI event loop hiçbir zaman bloklanmamalı.
- **Terminal restore:** Panic durumunda bile terminal state restore edilmeli (panic hook kullan).

## Dikkat Edilecekler

- samtools stats ve bcftools stats formatları benzer ama farklı. SN section'ındaki field index'leri farklıdır.
- bcftools stats DP section'ında bin değerleri string olabilir (`>500` gibi). `DpEntry.bin` tipi `String`.
- FastQC zip arşivleri içinde `*/fastqc_data.txt` yolu değişkenlik gösterebilir, `ends_with` ile aranır.
- `##FastQC` header satırı `>>` ile başlamaz, `#` ile başlar — section parser'da dikkat.
- ratatui her frame'de tüm UI'ı yeniden çizer (immediate mode). State'i UI'dan ayır. `draw()` her zaman `&AppState` alır, state mutasyonu sadece `update()` içinde yapılır.
- Splash screen: Başlangıçta animasyonlu ASCII logo + spark partikülleri gösterilir. `splash_tick` 250ms tick ile artırılır (`Action::Render` handler'ında), 8 tick (~2sn) + veri hazır olunca ana UI'a geçilir. `pending_results` ile veri splash bitene kadar buffer'lanır.
- crossterm event-stream feature'ı futures StreamExt gerektirir.
- `--generate` modu stats dosyası / `_fastqc.zip` zaten varsa skip eder (idempotent).
- Generator, samtools/bcftools/fastqc'nin PATH'de olup olmadığını kontrol eder.
- FastQC çıktı isimlendirmesi: `sample.fastq.gz` → `sample_fastqc.zip`. FastQC kendi disk'e yazar, stdout capture gerekmez.
- Overview tab'da `OverviewRow` struct'ı ile sort/filter altyapısı var. Yeni sütun eklemek: `build_overview_rows()` ve `SortColumn` enum'unu güncelle.
- Search modu `Arc<AtomicBool>` ile EventHandler ve AppState arasında paylaşılır (async tokio task erişimi için).
- CSV export `Option<T>` field'ları kullanır, csv crate None'ı boş string olarak yazar.
- `--export-json` ve `--export-csv` birlikte kullanılabilir, veri bir kez yüklenir.
- Summary tab `h`/`l` ile horizontal scroll destekler (frozen File sütunu + kaydırılabilir metrik sütunları).
- `SummarySortColumn` enum 17 sütun üzerinde sort destekler, `s` tuşu context-aware: Summary tab'da SummarySortColumn, diğerlerinde SortColumn cycle eder.
- ThresholdConfig TOML'dan yüklenebilir, `#[derive(Deserialize)]` ile. Default değerler biyoinformatik standartlarına göre.
- GC% threshold'u `gc_deviation` olarak uygulanır: `abs(gc - 50.0)` hesaplanıp LowerIsBetter ile değerlendirilir.
- `--strict` modu export sonrası `check_qc_failures()` ile tüm sample'ları evaluate eder, FAIL varsa exit(1).
- CSV export threshold'lar verildiğinde `qc_status` sütunu ekler (`#[serde(skip_serializing_if = "Option::is_none")]`).
