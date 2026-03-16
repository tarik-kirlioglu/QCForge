use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{QcForgeError, Result};
use crate::parser::types::{
    AfEntry, BcftoolsStats, BcftoolsSummary, DpEntry, IddEntry, QualEntry, StEntry, TsTvStats,
};

pub fn parse_bcftools_stats(path: &Path, content: &str) -> Result<BcftoolsStats> {
    let mut raw = BTreeMap::new();
    let mut tstv = TsTvStats::default();
    let mut sub_types = Vec::new();
    let mut allele_freq = Vec::new();
    let mut qual_dist = Vec::new();
    let mut indel_dist = Vec::new();
    let mut depth_dist = Vec::new();

    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            continue;
        }

        match fields[0] {
            "SN" if fields.len() >= 4 => {
                // SN\tid\tkey:\tvalue
                let key = fields[2].trim().trim_end_matches(':').to_string();
                let value = fields[3].trim().to_string();
                raw.insert(key, value);
            }
            "TSTV" if fields.len() >= 8 => {
                // TSTV\tid\tts\ttv\tts/tv\tts (1st ALT)\ttv (1st ALT)\tts/tv (1st ALT)
                tstv = TsTvStats {
                    ts: parse_num(path, "TSTV.ts", fields[2])?,
                    tv: parse_num(path, "TSTV.tv", fields[3])?,
                    ts_tv_ratio: parse_num(path, "TSTV.ratio", fields[4])?,
                    ts_first_alt: parse_num(path, "TSTV.ts_1st", fields[5])?,
                    tv_first_alt: parse_num(path, "TSTV.tv_1st", fields[6])?,
                    ts_tv_ratio_first_alt: parse_num(path, "TSTV.ratio_1st", fields[7])?,
                };
            }
            "ST" if fields.len() >= 4 => {
                // ST\tid\ttype\tcount
                sub_types.push(StEntry {
                    sub_type: fields[2].trim().to_string(),
                    count: parse_num(path, "ST.count", fields[3])?,
                });
            }
            "AF" if fields.len() >= 5 => {
                // AF\tid\tallele_freq\tnum_snps\tnum_indels
                allele_freq.push(AfEntry {
                    allele_freq: parse_num(path, "AF.freq", fields[2])?,
                    num_snps: parse_num(path, "AF.snps", fields[3])?,
                    num_indels: parse_num(path, "AF.indels", fields[4])?,
                });
            }
            "QUAL" if fields.len() >= 6 => {
                // QUAL\tid\tquality\tnum_snps\tnum_indels\tnum_tstv
                qual_dist.push(QualEntry {
                    quality: parse_num(path, "QUAL.quality", fields[2])?,
                    num_snps: parse_num(path, "QUAL.snps", fields[3])?,
                    num_indels: parse_num(path, "QUAL.indels", fields[4])?,
                    num_tstv: parse_num(path, "QUAL.tstv", fields[5])?,
                });
            }
            "IDD" if fields.len() >= 4 => {
                // IDD\tid\tlength\tcount
                indel_dist.push(IddEntry {
                    length: parse_num(path, "IDD.length", fields[2])?,
                    count: parse_num(path, "IDD.count", fields[3])?,
                });
            }
            "DP" if fields.len() >= 7 => {
                // DP\tid\tbin\tnum_genotypes\tfrac_genotypes\tnum_sites\tfrac_sites
                depth_dist.push(DpEntry {
                    bin: parse_num(path, "DP.bin", fields[2])?,
                    num_genotypes: parse_num(path, "DP.num_gt", fields[3])?,
                    frac_genotypes: parse_num(path, "DP.frac_gt", fields[4])?,
                    num_sites: parse_num(path, "DP.num_sites", fields[5])?,
                    frac_sites: parse_num(path, "DP.frac_sites", fields[6])?,
                });
            }
            _ => {} // skip unknown sections
        }
    }

    let summary = build_summary(raw);

    Ok(BcftoolsStats {
        source_file: path.to_path_buf(),
        summary,
        tstv,
        substitution_types: sub_types,
        allele_freq,
        qual_dist,
        indel_dist,
        depth_dist,
    })
}

fn build_summary(raw: BTreeMap<String, String>) -> BcftoolsSummary {
    let get_u64 = |key: &str| -> u64 {
        raw.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };

    BcftoolsSummary {
        num_samples: get_u64("number of samples"),
        num_records: get_u64("number of records"),
        num_no_alts: get_u64("number of no-ALTs"),
        num_snps: get_u64("number of SNPs"),
        num_mnps: get_u64("number of MNPs"),
        num_indels: get_u64("number of indels"),
        num_others: get_u64("number of others"),
        num_multiallelic_sites: get_u64("number of multiallelic sites"),
        num_multiallelic_snp_sites: get_u64("number of multiallelic SNP sites"),
        raw,
    }
}

fn parse_num<T: std::str::FromStr>(path: &Path, field: &str, value: &str) -> Result<T> {
    value
        .trim()
        .parse::<T>()
        .map_err(|_| QcForgeError::NumericParse {
            field: format!("{}:{}", path.display(), field),
            value: value.trim().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE_BCFTOOLS: &str = "\
# This file was produced by bcftools stats (1.17+htslib-1.17) and target_regions.bed
# The command line was: bcftools stats input.vcf.gz
# SN, Summary numbers:
#   number of records, number of no-ALTs, number of SNPs, number of MNPs, number of indels, number of others
SN\t0\tnumber of samples:\t1
SN\t0\tnumber of records:\t8500
SN\t0\tnumber of no-ALTs:\t0
SN\t0\tnumber of SNPs:\t7200
SN\t0\tnumber of MNPs:\t0
SN\t0\tnumber of indels:\t1250
SN\t0\tnumber of others:\t50
SN\t0\tnumber of multiallelic sites:\t320
SN\t0\tnumber of multiallelic SNP sites:\t180
# TSTV, transitions/transversions:
#   [2]id\t[3]ts\t[4]tv\t[5]ts/tv\t[6]ts (1st ALT)\t[7]tv (1st ALT)\t[8]ts/tv (1st ALT)
TSTV\t0\t4800\t2400\t2.00\t4600\t2300\t2.00
# Substitution types:
# ST, Type and target:
#   [2]id\t[3]type\t[4]count
ST\t0\tA>C\t380
ST\t0\tA>G\t1200
ST\t0\tA>T\t290
ST\t0\tC>A\t410
ST\t0\tC>G\t350
ST\t0\tC>T\t1850
ST\t0\tG>A\t1800
ST\t0\tG>C\t340
ST\t0\tG>T\t400
ST\t0\tT>A\t280
ST\t0\tT>C\t1180
ST\t0\tT>G\t370
# Allele frequency distribution:
# AF, Stats by non-reference allele frequency:
#   [2]id\t[3]allele frequency\t[4]number of SNPs\t[5]number of indels
AF\t0\t0.000000\t0\t0
AF\t0\t0.100000\t1200\t250
AF\t0\t0.200000\t800\t180
AF\t0\t0.300000\t500\t120
AF\t0\t0.400000\t400\t100
AF\t0\t0.500000\t2500\t350
AF\t0\t0.600000\t380\t80
AF\t0\t0.700000\t320\t50
AF\t0\t0.800000\t500\t60
AF\t0\t0.900000\t300\t30
AF\t0\t1.000000\t300\t30
# Quality distribution:
# QUAL, Stats by quality:
#   [2]id\t[3]quality\t[4]number of SNPs\t[5]number of indels\t[6]number of ts/tv
QUAL\t0\t10.0\t500\t100\t1.5
QUAL\t0\t20.0\t800\t150\t1.8
QUAL\t0\t30.0\t1200\t200\t2.0
QUAL\t0\t50.0\t2000\t350\t2.1
QUAL\t0\t100.0\t1500\t250\t2.0
QUAL\t0\t200.0\t1000\t150\t2.0
QUAL\t0\t500.0\t200\t50\t1.9
# InDel length distribution:
# IDD, InDel distribution:
#   [2]id\t[3]length (deletions negative)\t[4]count
IDD\t0\t-10\t15
IDD\t0\t-5\t45
IDD\t0\t-4\t60
IDD\t0\t-3\t85
IDD\t0\t-2\t120
IDD\t0\t-1\t280
IDD\t0\t1\t310
IDD\t0\t2\t130
IDD\t0\t3\t75
IDD\t0\t4\t50
IDD\t0\t5\t35
IDD\t0\t10\t10
# Depth distribution:
# DP, Pair-wise calculated depth distribution (HWE):
#   [2]id\t[3]bin\t[4]number of genotypes\t[5]fraction of genotypes (%)\t[6]number of sites\t[7]fraction of sites (%)
DP\t0\t0\t50\t0.59\t50\t0.59
DP\t0\t5\t200\t2.35\t200\t2.35
DP\t0\t10\t800\t9.41\t800\t9.41
DP\t0\t15\t1500\t17.65\t1500\t17.65
DP\t0\t20\t2200\t25.88\t2200\t25.88
DP\t0\t25\t1800\t21.18\t1800\t21.18
DP\t0\t30\t1200\t14.12\t1200\t14.12
DP\t0\t35\t500\t5.88\t500\t5.88
DP\t0\t40\t200\t2.35\t200\t2.35
DP\t0\t50\t50\t0.59\t50\t0.59";

    #[test]
    fn test_parse_summary() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.summary.num_samples, 1);
        assert_eq!(result.summary.num_records, 8500);
        assert_eq!(result.summary.num_snps, 7200);
        assert_eq!(result.summary.num_indels, 1250);
        assert_eq!(result.summary.num_multiallelic_sites, 320);
    }

    #[test]
    fn test_parse_tstv() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.tstv.ts, 4800);
        assert_eq!(result.tstv.tv, 2400);
        assert!((result.tstv.ts_tv_ratio - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_substitution_types() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.substitution_types.len(), 12);
        assert_eq!(result.substitution_types[0].sub_type, "A>C");
        assert_eq!(result.substitution_types[0].count, 380);
        assert_eq!(result.substitution_types[5].sub_type, "C>T");
        assert_eq!(result.substitution_types[5].count, 1850);
    }

    #[test]
    fn test_parse_allele_freq() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.allele_freq.len(), 11);
        assert!((result.allele_freq[4].allele_freq - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_parse_indel_dist() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.indel_dist.len(), 12);
        assert_eq!(result.indel_dist[0].length, -10);
        assert_eq!(result.indel_dist[5].length, -1);
        assert_eq!(result.indel_dist[5].count, 280);
        assert_eq!(result.indel_dist[6].length, 1);
        assert_eq!(result.indel_dist[6].count, 310);
    }

    #[test]
    fn test_parse_depth_dist() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        assert_eq!(result.depth_dist.len(), 10);
        assert_eq!(result.depth_dist[4].bin, 20);
        assert_eq!(result.depth_dist[4].num_genotypes, 2200);
    }

    #[test]
    fn test_snp_percent() {
        let path = PathBuf::from("test.vcf.stats");
        let result = parse_bcftools_stats(&path, SAMPLE_BCFTOOLS).unwrap();

        let pct = result.summary.snp_percent();
        assert!((pct - 84.7).abs() < 0.1);
    }
}
