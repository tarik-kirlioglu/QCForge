# Parser Module

QC araç çıktılarını parse eden modüller. 3 parser, toplam 18 unit test.

## Desteklenen Formatlar

### samtools stats (`parser/samtools.rs`)
- Tab-separated, satır bazlı format
- Parse edilen section'lar: SN (Summary Numbers), COV, GCF, GCL, IS, RL
- `#` ile başlayan satırlar yorum/header
- SN formatı: `SN\tkey:\tvalue\t# comment`
- Key'deki trailing `:` strip edilmeli
- Dosya tespiti: İlk satırlarda "This file was produced by samtools stats" aranır
- 5 unit test

### bcftools stats (`parser/bcftools.rs`)
- Benzer tab-separated format
- Parse edilen section'lar: SN, TSTV, ST, AF, QUAL, IDD, DP
- SN formatı: `SN\tid\tkey:\tvalue` (samtools'dan farklı: id field'i var)
- DP.bin alanı String tipinde (`>500` gibi non-numeric değerler olabilir)
- QUAL.num_tstv alanı f64 tipinde (1.5 gibi ondalıklı değerler)
- Dosya tespiti: İlk satırlarda "This file was produced by bcftools stats" aranır
- 7 unit test

### FastQC (`parser/fastqc.rs`)
- Zip arşivinden çıkarılır (`*_fastqc.zip` → `*/fastqc_data.txt`)
- Section format: `>>Module Name\tpass|warn|fail` ... `>>END_MODULE`
- `#` satırları kolon header'ları
- `##FastQC` header satırı section olarak algılanmamalı
- Parse edilen modüller: Basic Statistics, Per base sequence quality, Per sequence quality scores, Per base sequence content, Per sequence GC content, Sequence Length Distribution, Overrepresented sequences
- Zip extraction `spawn_blocking` ile yapılır (CPU-bound)
- 6 unit test

## Kod Kuralları

- Her parser fonksiyonu `Result<T, QcForgeError>` döndürmeli
- Parse hataları `QcForgeError::NumericParse` ile, dosya adı ve field bilgisi dahil
- Bilinmeyen section tag'leri sessizce skip edilmeli (forward compatibility)
- Raw key-value pair'leri her zaman `BTreeMap<String, String>` olarak da saklanmalı (gelecekte yeni field desteği için)
- Unit test'lerde gerçek samtools/bcftools/FastQC çıktılarından alınmış küçük örnekler kullan (inline string)
