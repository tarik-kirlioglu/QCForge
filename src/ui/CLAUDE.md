# UI Module

ratatui ile terminal render katmanı.

## Yapı

```
ui/
├── mod.rs        # Root render fn: draw(frame, app_state)
├── layout.rs     # Header, tab bar, content area, footer layout bölümleri
├── tabs/         # Her tab için ayrı render modülü
│   ├── overview.rs   # Tüm QC verilerinin özet dashboard'u
│   ├── samtools.rs   # samtools stats detay görünümü
│   ├── bcftools.rs   # bcftools stats detay görünümü
│   └── fastqc.rs     # FastQC detay görünümü
└── widgets/      # Tekrar kullanılabilir widget wrapper'ları
    ├── gauge.rs  # Yüzdelik gauge (mapping %, dup %)
    ├── chart.rs  # Line/bar chart wrappers
    └── table.rs  # Styled table helpers
```

## Renk Şeması

| Durum | Renk | Kullanım |
|-------|------|----------|
| PASS / İyi | Green | mapping > 90%, Q >= 28 |
| WARN / Marjinal | Yellow | mapping 80-90%, Q 20-28 |
| FAIL / Kötü | Red | mapping < 80%, Q < 20 |
| Header/Border | Cyan | Çerçeve ve başlıklar |
| Normal text | White | Veri gösterimi |
| Secondary | Gray | Yorum, açıklama |

## Layout Kuralları

- ratatui immediate mode: her frame'de UI tamamen yeniden çizilir
- State UI fonksiyonlarına `&AppState` referansı ile geçirilir, UI fonksiyonları state'i değiştirmez
- Layout `ratatui::layout::Layout` ile constraint-based yapılır
- Tab bar'da aktif tab highlight edilir (reverse style)
- Footer'da context-sensitive keybinding bilgileri gösterilir
- Terminal boyutu değiştiğinde otomatik re-layout (Resize event ile)

## Widget Kullanımı

- `Gauge`: Yüzdelik değerler için (mapping rate, duplication rate)
- `Table`: Key-value summary verileri için (SN section)
- `BarChart`: Dağılım verileri için (insert size, indel length)
- `Chart` + `Dataset`: Line chart (coverage, GC content, quality scores)
- `Paragraph`: Metin bilgileri, help overlay
- `Tabs`: Üst tab bar widget'ı
