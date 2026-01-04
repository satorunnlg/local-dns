use crate::db::Record;
use hickory_server::proto::rr::{Name, RData, Record as DnsRecord, RecordType};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use tracing::warn;

/// DNSレコードを構築
pub fn build_dns_record(
    query_name: &Name,
    record: &Record,
) -> Option<DnsRecord> {
    let ttl = record.ttl as u32;

    match record.record_type.as_str() {
        "A" => {
            // IPv4アドレスをパース
            match Ipv4Addr::from_str(&record.content) {
                Ok(ip) => {
                    let rdata = RData::A(ip.into());
                    Some(DnsRecord::from_rdata(
                        query_name.clone(),
                        ttl,
                        rdata,
                    ))
                }
                Err(e) => {
                    warn!(
                        "IPv4アドレスのパースに失敗: {} ({})",
                        record.content, e
                    );
                    None
                }
            }
        }
        "AAAA" => {
            // IPv6アドレスをパース
            match Ipv6Addr::from_str(&record.content) {
                Ok(ip) => {
                    let rdata = RData::AAAA(ip.into());
                    Some(DnsRecord::from_rdata(
                        query_name.clone(),
                        ttl,
                        rdata,
                    ))
                }
                Err(e) => {
                    warn!(
                        "IPv6アドレスのパースに失敗: {} ({})",
                        record.content, e
                    );
                    None
                }
            }
        }
        "CNAME" => {
            // CNAMEターゲットをパース
            match Name::from_str(&record.content) {
                Ok(target) => {
                    use hickory_server::proto::rr::rdata::CNAME;
                    let cname = CNAME(target);
                    let rdata = RData::CNAME(cname);
                    Some(DnsRecord::from_rdata(
                        query_name.clone(),
                        ttl,
                        rdata,
                    ))
                }
                Err(e) => {
                    warn!(
                        "CNAME ターゲットのパースに失敗: {} ({})",
                        record.content, e
                    );
                    None
                }
            }
        }
        _ => {
            warn!("サポートされていないレコードタイプ: {}", record.record_type);
            None
        }
    }
}

/// RecordTypeを文字列に変換（将来のロギング拡張用）
#[allow(dead_code)]
pub fn record_type_to_string(rt: RecordType) -> String {
    match rt {
        RecordType::A => "A".to_string(),
        RecordType::AAAA => "AAAA".to_string(),
        RecordType::CNAME => "CNAME".to_string(),
        _ => format!("{:?}", rt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Record as DbRecord;

    #[test]
    fn test_build_a_record() {
        let query_name = Name::from_str("app.local.test").unwrap();
        let record = DbRecord {
            id: 1,
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
            active: 1,
        };

        let dns_record = build_dns_record(&query_name, &record).unwrap();
        assert_eq!(dns_record.name(), &query_name);
        assert_eq!(dns_record.ttl(), 60);

        if let RData::A(ip) = dns_record.data() {
            assert_eq!(ip.to_string(), "127.0.0.1");
        } else {
            panic!("Expected A record");
        }
    }

    #[test]
    fn test_build_aaaa_record() {
        let query_name = Name::from_str("app.local.test").unwrap();
        let record = DbRecord {
            id: 1,
            domain_pattern: "app.local.test".to_string(),
            record_type: "AAAA".to_string(),
            content: "::1".to_string(),
            ttl: 60,
            active: 1,
        };

        let dns_record = build_dns_record(&query_name, &record).unwrap();

        if let RData::AAAA(ip) = dns_record.data() {
            assert_eq!(ip.to_string(), "::1");
        } else {
            panic!("Expected AAAA record");
        }
    }

    #[test]
    fn test_build_cname_record() {
        let query_name = Name::from_str("alias.local.test").unwrap();
        let record = DbRecord {
            id: 1,
            domain_pattern: "alias.local.test".to_string(),
            record_type: "CNAME".to_string(),
            content: "target.local.test".to_string(),
            ttl: 60,
            active: 1,
        };

        let dns_record = build_dns_record(&query_name, &record).unwrap();

        if let RData::CNAME(cname) = dns_record.data() {
            // hickory-serverのCNAMEは末尾にドットを付けない
            let target_str = cname.0.to_string();
            assert!(
                target_str == "target.local.test" || target_str == "target.local.test.",
                "Expected target.local.test or target.local.test., got {}", target_str
            );
        } else {
            panic!("Expected CNAME record");
        }
    }

    #[test]
    fn test_build_invalid_a_record() {
        let query_name = Name::from_str("app.local.test").unwrap();
        let record = DbRecord {
            id: 1,
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "invalid-ip".to_string(),
            ttl: 60,
            active: 1,
        };

        let dns_record = build_dns_record(&query_name, &record);
        assert!(dns_record.is_none());
    }
}
