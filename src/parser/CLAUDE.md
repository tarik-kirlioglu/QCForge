# Parser Module

Modules for parsing QC tool outputs. 3 parsers, 18 unit tests total.

## Supported Formats

### samtools stats (`parser/samtools.rs`)
- Tab-separated, line-based format
- Parsed sections: SN (Summary Numbers), COV, GCF, GCL, IS, RL
- Lines starting with `#` are comments/headers
- SN format: `SN\tkey:\tvalue\t# comment`
- Trailing `:` in keys must be stripped
- File detection: first lines are searched for "This file was produced by samtools stats"
- 5 unit tests

### bcftools stats (`parser/bcftools.rs`)
- Similar tab-separated format
- Parsed sections: SN, TSTV, ST, AF, QUAL, IDD, DP
- SN format: `SN\tid\tkey:\tvalue` (different from samtools: has an id field)
- DP.bin field is String type (can contain non-numeric values like `>500`)
- QUAL.num_tstv field is f64 type (decimal values like 1.5)
- File detection: first lines are searched for "This file was produced by bcftools stats"
- 7 unit tests

### FastQC (`parser/fastqc.rs`)
- Extracted from zip archive (`*_fastqc.zip` → `*/fastqc_data.txt`)
- Section format: `>>Module Name\tpass|warn|fail` ... `>>END_MODULE`
- `#` lines are column headers
- `##FastQC` header line must not be detected as a section
- Parsed modules: Basic Statistics, Per base sequence quality, Per sequence quality scores, Per base sequence content, Per sequence GC content, Sequence Length Distribution, Overrepresented sequences
- Zip extraction uses `spawn_blocking` (CPU-bound)
- 6 unit tests

## Code Rules

- Every parser function must return `Result<T, QcForgeError>`
- Parse errors use `QcForgeError::NumericParse` with filename and field info included
- Unknown section tags must be silently skipped (forward compatibility)
- Raw key-value pairs should always be stored as `BTreeMap<String, String>` as well (for future field support)
- Unit tests should use small samples taken from real samtools/bcftools/FastQC outputs (inline strings)
