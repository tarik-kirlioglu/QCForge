# UI Module

ratatui ile terminal render katmanı. 4 tab, help overlay, loading/error state'leri.

## Yapı

```
ui/
├── mod.rs        # Root render fn: draw(frame, app_state) + loading/error/help overlay
├── layout.rs     # Header, tab bar, content area, footer layout bölümleri
├── tabs/
│   ├── overview.rs   # Tüm QC verilerinin özet dashboard'u (dosya listesi, aggregate stats, gauge'lar)
│   ├── samtools.rs   # samtools stats detay: summary table + mapping/dup/paired gauge'ları
│   ├── bcftools.rs   # bcftools stats detay: summary + Ts/Tv + substitution types + indel distribution
│   └── fastqc.rs     # FastQC detay: basic stats + module status (pass/warn/fail) + per-base quality chart
└── widgets/
    ├── gauge.rs  # Quality renk hesaplama (mapping_style, duplication_style)
    └── table.rs  # Styled table helpers (header_style, highlight_style)
```

## Tabs

| Tab | İçerik |
|-----|--------|
| Summary | MultiQC-tarzı geniş tablo: tüm sample'lar + tüm metrikler, threshold renkli hücreler, h/l horizontal scroll |
| Overview | Dosya sayıları, aggregate stats, avg mapping/dup gauge, sıralanabilir+filtrelenebilir dosya listesi |
| samtools | Summary Numbers tablosu + Mapping/Duplication/Properly Paired gauge'ları |
| bcftools | Summary + Ts/Tv + Substitution Types (inline bar, Ts=cyan Tv=magenta) + InDel Distribution (del=red ins=green) |
| FastQC | Basic Statistics + Module Status (renkli PASS/WARN/FAIL) + Per Base Quality (bar chart) |

## Renk Şeması

| Durum | Renk | Kullanım |
|-------|------|----------|
| PASS / İyi | Green | mapping > 90%, Q >= 28, dup <= 10% |
| WARN / Marjinal | Yellow | mapping 80-90%, Q 20-28, dup 10-20% |
| FAIL / Kötü | Red | mapping < 80%, Q < 20, dup > 20% |
| Header/Border | Cyan | Çerçeve ve başlıklar |
| Transitions | Cyan | bcftools substitution types (A>G, G>A, C>T, T>C) |
| Transversions | Magenta | bcftools substitution types (diğerleri) |
| Insertions | Green | bcftools indel dist (pozitif length) |
| Deletions | Red | bcftools indel dist (negatif length) |
| Normal text | White | Veri gösterimi |
| Secondary | Gray / DarkGray | Yorum, açıklama, ikincil bilgi |

## Layout Kuralları

- ratatui immediate mode: her frame'de UI tamamen yeniden çizilir
- State UI fonksiyonlarına `&AppState` referansı ile geçirilir, UI fonksiyonları state'i değiştirmez
- Layout `ratatui::layout::Layout` ile constraint-based yapılır
- Tab bar'da aktif tab highlight edilir (underlined + bold + cyan)
- Footer'da keybinding bilgileri gösterilir; search modu aktifken footer search bar'a dönüşür
- Overview tab'da `OverviewRow` struct'ı ile dosya listesi oluşturulur, sort → filter → render sırasıyla
- Aktif sort sütunu header'da `▲`/`▼` göstergesi ile belirtilir
- Aktif filtre footer'da `[filter: xxx]` ile gösterilir
- Her tab'da dosya header'ı: `tool: filename [n/total] n:Next p:Prev`
- Help overlay `?` tuşuyla toggle, centered_rect ile ortaya yerleşir

## Keybindings

- `q` / `Esc`: Quit (search modunda Esc filtreyi temizler)
- `←` / `→` / `Tab`: Tab değiştir
- `j` / `k` / `↑` / `↓`: Scroll
- `n` / `p`: Dosyalar arası geçiş
- `s`: Sort sütununu değiştir (context-aware: Overview vs Summary)
- `S`: Sort yönünü değiştir (asc/desc)
- `/`: Arama moduna gir (real-time filtreleme, Enter onayla, Esc temizle)
- `h` / `l`: Summary tab'da sütunları yatay kaydır
- `Ctrl+C`: Quit
- `?`: Help overlay toggle
