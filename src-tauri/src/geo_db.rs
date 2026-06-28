use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use log::{error, info, warn};
use tokio::sync::RwLock;

/// GeoIP database reader using MaxMind GeoLite2
pub struct GeoDatabase {
    mmdb_reader: Option<maxminddb::Reader<Vec<u8>>>,
    /// Cache for DNS lookups: domain -> Vec<IpAddr>
    dns_cache: Arc<RwLock<HashMap<String, Vec<IpAddr>>>>,
    /// Cache for geoip lookups: ip -> country_code
    geoip_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl GeoDatabase {
    pub fn new(geo_dir: &PathBuf) -> Self {
        let mmdb_path = geo_dir.join("GeoLite2-Country.mmdb");
        let mmdb_reader: Option<maxminddb::Reader<Vec<u8>>> = if mmdb_path.exists() {
            match std::fs::read(&mmdb_path) {
                Ok(data) => {
                    let size = data.len();
                    match maxminddb::Reader::from_source(data) {
                        Ok(reader) => {
                            info!("GeoIP database loaded: {:?} ({} bytes)", mmdb_path, size);
                            Some(reader)
                        }
                        Err(e) => {
                            warn!("Failed to parse GeoIP database {:?}: {e}", mmdb_path);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read GeoIP database {:?}: {e}", mmdb_path);
                    None
                }
            }
        } else {
            warn!("GeoIP database not found at {:?}. GeoIP lookups disabled.", mmdb_path);
            None
        };

        Self {
            mmdb_reader,
            dns_cache: Arc::new(RwLock::new(HashMap::new())),
            geoip_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Look up the country code for an IP address
    pub fn lookup_ip(&self, ip: &IpAddr) -> Option<String> {
        let reader = self.mmdb_reader.as_ref()?;
        match reader.lookup::<maxminddb::geoip2::Country>(*ip) {
            Ok(country) => country
                .country
                .and_then(|c| c.iso_code)
                .map(|code| code.to_string()),
            Err(e) => {
                error!("GeoIP lookup failed for {ip}: {e}");
                None
            }
        }
    }

    /// Resolve a domain to IPs (DNS lookup)
    pub async fn resolve_domain(&self, domain: &str) -> Vec<IpAddr> {
        // Check cache first
        {
            let cache = self.dns_cache.read().await;
            if let Some(addrs) = cache.get(domain) {
                return addrs.clone();
            }
        }

        // Perform DNS lookup
        match tokio::net::lookup_host(format!("{domain}:0")).await {
            Ok(addrs) => {
                let ips: Vec<IpAddr> = addrs.map(|addr| addr.ip()).collect();
                // Cache the result
                let mut cache = self.dns_cache.write().await;
                cache.insert(domain.to_string(), ips.clone());
                // Limit cache size
                if cache.len() > 1000 {
                    cache.clear();
                }
                ips
            }
            Err(e) => {
                error!("DNS lookup failed for {domain}: {e}");
                Vec::new()
            }
        }
    }

    /// Check if a domain belongs to a given country code (geoip)
    pub async fn domain_in_country(&self, domain: &str, country_code: &str) -> bool {
        let ips = self.resolve_domain(domain).await;
        for ip in &ips {
            // Check cached geoip
            let cached = {
                let cache = self.geoip_cache.read().await;
                cache.get(&ip.to_string()).cloned()
            };
            let country = if let Some(c) = cached {
                c
            } else {
                let c = self.lookup_ip(ip).unwrap_or_default();
                // Cache it
                let mut cache = self.geoip_cache.write().await;
                cache.insert(ip.to_string(), c.clone());
                if cache.len() > 5000 {
                    cache.clear();
                }
                c
            };
            if country.eq_ignore_ascii_case(country_code) {
                return true;
            }
        }
        false
    }

    /// Check if a domain matches a geosite category
    /// Uses a simplified built-in category map for common categories
    pub async fn domain_in_geosite(&self, domain: &str, category: &str) -> bool {
        let category_lower = category.to_lowercase();

        // Built-in common geosite categories (simplified)
        match category_lower.as_str() {
            "google" => self.matches_domain_suffix(domain, &[
                "google.com", "googleapis.com", "gstatic.com", "googleusercontent.com",
                "googleadservices.com", "google-analytics.com", "googlevideo.com",
                "youtube.com", "ytimg.com", "ggpht.com", "googlemail.com", "gmail.com",
                "googleplex.com", "blogspot.com", "blogger.com",
            ]),
            "youtube" => self.matches_domain_suffix(domain, &[
                "youtube.com", "ytimg.com", "youtu.be", "googlevideo.com",
                "youtube-nocookie.com", "yt.be",
            ]),
            "netflix" => self.matches_domain_suffix(domain, &[
                "netflix.com", "nflxvideo.net", "nflxext.com", "nflximg.com",
                "nflxso.net", "nflxcdn.com", "netflix.net",
            ]),
            "spotify" => self.matches_domain_suffix(domain, &[
                "spotify.com", "spotify.net", "scdn.co", "spoti.fi",
            ]),
            "telegram" => self.matches_domain_suffix(domain, &[
                "telegram.org", "t.me", "tdesktop.com", "telegram.me",
                "telesco.pe", "telegram.dog",
            ]),
            "twitter" | "x" => self.matches_domain_suffix(domain, &[
                "twitter.com", "x.com", "t.co", "twimg.com", "twitpic.com",
            ]),
            "facebook" => self.matches_domain_suffix(domain, &[
                "facebook.com", "fbcdn.net", "fb.com", "messenger.com",
                "instagram.com", "cdninstagram.com", "whatsapp.com",
            ]),
            "apple" => self.matches_domain_suffix(domain, &[
                "apple.com", "icloud.com", "apple-cloudkit.com", "aaplimg.com",
                "appstore.com", "itunes.com", "icloud-content.com",
            ]),
            "microsoft" => self.matches_domain_suffix(domain, &[
                "microsoft.com", "msn.com", "live.com", "office.com",
                "office365.com", "azure.com", "windows.com", "bing.com",
                "outlook.com", "hotmail.com", "onenote.com",
            ]),
            "cloudflare" => self.matches_domain_suffix(domain, &[
                "cloudflare.com", "cloudflare.net", "cloudflare-dns.com",
                "cloudflarestream.com", "cloudflareinsights.com",
            ]),
            "openai" | "chatgpt" => self.matches_domain_suffix(domain, &[
                "openai.com", "chatgpt.com", "oaistatic.com", "oaiusercontent.com",
                "api.openai.com", "cdn.openai.com",
            ]),
            "category-ads" | "category-ads-all" => self.matches_domain_suffix(domain, &[
                "doubleclick.net", "googlesyndication.com", "googleadservices.com",
                "google-analytics.com", "googletagmanager.com", "adservice.google.com",
                "adsrvr.org", "adnxs.com", "rubiconproject.com", "pubmatic.com",
                "criteo.com", "criteo.net", "scorecardresearch.com",
            ]),
            "category-bank" => self.matches_domain_suffix(domain, &[
                "bankofamerica.com", "chase.com", "wellsfargo.com", "citi.com",
                "capitalone.com", "usbank.com", "hsbc.com", "barclays.com",
                "jpmorgan.com", "amex.com", "americanexpress.com",
            ]),
            "category-social-media" => self.matches_domain_suffix(domain, &[
                "facebook.com", "twitter.com", "x.com", "instagram.com",
                "linkedin.com", "reddit.com", "tiktok.com", "snapchat.com",
                "pinterest.com", "tumblr.com", "discord.com", "discord.gg",
                "telegram.org", "whatsapp.com", "wechat.com", "qq.com",
            ]),
            "category-video-streaming" => self.matches_domain_suffix(domain, &[
                "youtube.com", "netflix.com", "hulu.com", "disneyplus.com",
                "hbomax.com", "primevideo.com", "twitch.tv", "vimeo.com",
                "dailymotion.com", "peacocktv.com", "paramountplus.com",
            ]),
            "category-games" => self.matches_domain_suffix(domain, &[
                "steampowered.com", "steamcommunity.com", "epicgames.com",
                "blizzard.com", "battle.net", "origin.com", "ea.com",
                "ubisoft.com", "rockstargames.com", "xbox.com", "playstation.com",
                "nintendo.com", "riotgames.com", "valvesoftware.com",
            ]),
            _ => {
                // For unknown categories, match by domain suffix (e.g., "geosite:netflix" checks if domain ends with netflix)
                self.matches_domain_suffix(domain, &[&format!("{}.com", category)])
            }
        }
    }

    fn matches_domain_suffix(&self, domain: &str, suffixes: &[&str]) -> bool {
        let domain = domain.trim_matches('.').to_lowercase();
        suffixes.iter().any(|suffix| {
            let suffix = suffix.trim_matches('.').to_lowercase();
            domain == suffix || domain.ends_with(&format!(".{suffix}"))
        })
    }
}
