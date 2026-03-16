# Parser Module

QC araç çıktılarını parse eden modüller.

## Desteklenen Formatlar

### samtools stats
- Tab-separated, satır bazlı format
- Section tag'leri: SN (Summary Numbers), COV, GCF, GCL, IS, RL, ID, FFQ, LFQ, GCC
- `#` ile başlayan satırlar yorum/header
- SN formatı: `SN\tkey:\tvalue\t# comment`
- Key'deki trailing `:` strip edilmeli
- Dosya tespiti: İlk satırlarda "This file was produced by samtools stats" aranır

### bcftools stats
- Benzer tab-separated format
- Section tag'leri: SN, TSTV, SiS, AF, QUAL, IDD, ST, DP
- SN formatı: `SN\tid\tkey\tvalue` (samtools'dan farklı: id field'i var)
- Dosya tespiti: İlk satırlarda "This file was produced by bcftools stats" aranır

### FastQC (fastqc_data.txt)
- Zip arşivinden çıkarılır (`*_fastqc.zip` → `*/fastqc_data.txt`)
- Section format: `>>Module Name\tpass|warn|fail` ... `>>END_MODULE`
- `#` satırları kolon header'ları
- Dosya tespiti: `*_fastqc.zip` glob pattern

## Kod Kuralları

- Her parser fonksiyonu `Result<T, QcForgeError>` döndürmeli
- Parse hataları `QcForgeError::ParseError` ile, dosya adı ve detay bilgisi dahil
- Bilinmeyen section tag'leri sessizce skip edilmeli (forward compatibility)
- Sayısal değerler parse edilemezse `QcForgeError::NumericParse` ile hata
- Raw key-value pair'leri her zaman `BTreeMap<String, String>` olarak da saklanmalı (gelecekte yeni field desteği için)
- Unit test'lerde gerçek samtools/bcftools/FastQC çıktılarından alınmış küçük örnekler kullan
