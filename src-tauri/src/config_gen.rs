use serde_json::{json, Value};

/// Generate a complete sing-box configuration from a node and route rules
pub fn generate_config(node: &Value, rules: &[Value], settings: &Value) -> Value {
    let protocol = node
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("direct");
    let outbound = generate_outbound(protocol, node);
    let dns_server = settings.get("dns_server").and_then(|v| v.as_str()).unwrap_or("1.1.1.1");
    let bypass_ru = settings.get("bypass_ru").and_then(|v| v.as_bool()).unwrap_or(true);
    let kill_switch = settings.get("kill_switch").and_then(|v| v.as_bool()).unwrap_or(false);
    let split_processes = settings.get("split_processes").and_then(|v| v.as_str()).unwrap_or("");
    let bypass_local = settings.get("bypass_local").and_then(|v| v.as_bool()).unwrap_or(true);
    let proxy_mode = settings.get("proxy_mode").and_then(|v| v.as_str()).unwrap_or("system");
    let dns = generate_dns(node_id, dns_server);

    // proxy_mode: "tunnel" = TUN (all traffic), "proxy" = HTTP/SOCKS proxy only
    let is_tunnel = proxy_mode == "tunnel";

    // Kill switch: override final outbound to block
    let final_outbound = if kill_switch { "block" } else { node_id };

    let mut default_route = serde_json::Map::new();
    default_route.insert("rules".to_string(), json!(generate_route_rules(rules, node_id, bypass_ru, bypass_local, split_processes)));
    default_route.insert("auto_detect_interface".to_string(), json!(true));
    default_route.insert("override_android_vpn".to_string(), json!(true));
    default_route.insert("final".to_string(), json!(final_outbound));

    // Generate inbounds based on mode
    let inbounds = if is_tunnel {
        json!([
            {
                "type": "tun",
                "tag": "tun-in",
                "interface_name": "tun0",
                "inet4_address": "172.19.0.1/30",
                "auto_route": true,
                "strict_route": true,
                "stack": "gvisor",
                "sniff": true,
                "sniff_override_destination": true,
                "domain_strategy": "prefer_ipv4",
                "mtu": 9000
            },
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 2080,
                "sniff": true,
                "sniff_override_destination": false,
                "domain_strategy": "prefer_ipv4"
            }
        ])
    } else {
        json!([
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 2080,
                "sniff": true,
                "sniff_override_destination": false,
                "domain_strategy": "prefer_ipv4"
            }
        ])
    };

    json!({
        "log": {
            "level": "info",
            "output": "",
            "timestamp": true
        },
        "inbounds": inbounds,
        "outbounds": [
            outbound,
            {
                "type": "direct",
                "tag": "direct"
            },
            {
                "type": "block",
                "tag": "block"
            },
            {
                "type": "dns",
                "tag": "dns-out"
            }
        ],
        "route": Value::Object(default_route),
        "dns": dns,
        "experimental": {
            "cache_file": {
                "enabled": true,
                "cache_id": "vpn-cache"
            }
        }
    })
}

fn generate_outbound(protocol: &str, node: &Value) -> Value {
    let server = node
        .get("server")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let port = node
        .get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(443) as u16;
    let default_config = json!({});
    let config = node.get("config").unwrap_or(&default_config);
    let tag = node.get("id").and_then(|v| v.as_str()).unwrap_or("proxy");

    match protocol {
        "vless" => generate_vless_outbound(tag, server, port, config),
        "vmess" => generate_vmess_outbound(tag, server, port, config),
        "trojan" => generate_trojan_outbound(tag, server, port, config),
        "shadowsocks" => generate_shadowsocks_outbound(tag, server, port, config),
        "hysteria2" => generate_hysteria2_outbound(tag, server, port, config),
        "wireguard" => generate_wireguard_outbound(tag, server, port, config),
        _ => json!({
            "type": "direct",
            "tag": tag
        }),
    }
}

fn generate_vless_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let uuid = config.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
    let flow = config.get("flow").and_then(|v| v.as_str()).unwrap_or("");
    let network = config.get("network").and_then(|v| v.as_str()).unwrap_or("tcp");
    let security = config.get("security").and_then(|v| v.as_str()).unwrap_or("none");
    let sni = config.get("sni").and_then(|v| v.as_str()).unwrap_or(server);

    let mut tls = json!(null);
    if security != "none" {
        tls = json!({
            "enabled": true,
            "server_name": sni,
            "utls": {
                "enabled": true,
                "fingerprint": config.get("fingerprint").and_then(|v| v.as_str()).unwrap_or("chrome")
            }
        });

        if security == "reality" {
            tls = json!({
                "enabled": true,
                "server_name": sni,
                "reality": {
                    "enabled": true,
                    "public_key": config.get("public_key").and_then(|v| v.as_str()).unwrap_or(""),
                    "short_id": config.get("short_id").and_then(|v| v.as_str()).unwrap_or("")
                },
                "utls": {
                    "enabled": true,
                    "fingerprint": config.get("fingerprint").and_then(|v| v.as_str()).unwrap_or("chrome")
                }
            });
        }
    }

    let mut transport = json!(null);
    if network == "ws" {
        transport = json!({
            "type": "ws",
            "path": config.get("path").and_then(|v| v.as_str()).unwrap_or("/"),
            "headers": {}
        });
    } else if network == "grpc" {
        transport = json!({
            "type": "grpc",
            "service_name": config.get("service_name").and_then(|v| v.as_str()).unwrap_or("")
        });
    }

    json!({
        "type": "vless",
        "tag": tag,
        "server": server,
        "server_port": port,
        "uuid": uuid,
        "flow": if flow.is_empty() { json!(null) } else { json!(flow) },
        "packet_encoding": "xudp",
        "tls": tls,
        "transport": transport
    })
}

fn generate_vmess_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let uuid = config.get("id").and_then(|v| v.as_str())
        .or_else(|| config.get("uuid").and_then(|v| v.as_str()))
        .unwrap_or("");
    let aid = config.get("aid").or_else(|| config.get("alterId")).and_then(|v| v.as_u64()).unwrap_or(0);
    let security = config.get("scy").or_else(|| config.get("security")).and_then(|v| v.as_str()).unwrap_or("auto");
    let net = config.get("net").or_else(|| config.get("network")).and_then(|v| v.as_str()).unwrap_or("tcp");
    let sni = config.get("sni").or_else(|| config.get("host")).and_then(|v| v.as_str()).unwrap_or(server);

    let mut tls = json!(null);
    let tls_val = config.get("tls").and_then(|v| v.as_str()).unwrap_or("none");
    if tls_val == "tls" {
        tls = json!({
            "enabled": true,
            "server_name": sni,
            "utls": {
                "enabled": true,
                "fingerprint": "chrome"
            }
        });
    }

    let mut transport = json!(null);
    if net == "ws" {
        transport = json!({
            "type": "ws",
            "path": config.get("path").and_then(|v| v.as_str()).unwrap_or("/"),
            "headers": {}
        });
    } else if net == "grpc" {
        transport = json!({
            "type": "grpc",
            "service_name": config.get("service_name").and_then(|v| v.as_str()).unwrap_or("")
        });
    } else if net == "kcp" {
        transport = json!({
            "type": "kcp"
        });
    } else if net == "http" || net == "h2" {
        transport = json!({
            "type": "http"
        });
    }

    json!({
        "type": "vmess",
        "tag": tag,
        "server": server,
        "server_port": port,
        "uuid": uuid,
        "alter_id": aid,
        "security": security,
        "packet_encoding": "xudp",
        "tls": tls,
        "transport": transport
    })
}

fn generate_trojan_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let password = config.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let sni = config.get("sni").and_then(|v| v.as_str()).unwrap_or(server);
    let allow_insecure = config.get("allow_insecure").and_then(|v| v.as_bool()).unwrap_or(false);

    json!({
        "type": "trojan",
        "tag": tag,
        "server": server,
        "server_port": port,
        "password": password,
        "tls": {
            "enabled": true,
            "server_name": sni,
            "insecure": allow_insecure,
            "utls": {
                "enabled": true,
                "fingerprint": "chrome"
            }
        }
    })
}

fn generate_shadowsocks_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let method = config.get("method").and_then(|v| v.as_str()).unwrap_or("aes-256-gcm");
    let password = config.get("password").and_then(|v| v.as_str()).unwrap_or("");

    json!({
        "type": "shadowsocks",
        "tag": tag,
        "server": server,
        "server_port": port,
        "method": method,
        "password": password
    })
}

fn generate_hysteria2_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let password = config.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let sni = config.get("sni").and_then(|v| v.as_str()).unwrap_or(server);
    let insecure = config.get("insecure").and_then(|v| v.as_bool()).unwrap_or(false);

    json!({
        "type": "hysteria2",
        "tag": tag,
        "server": server,
        "server_port": port,
        "password": password,
        "tls": {
            "enabled": true,
            "server_name": sni,
            "insecure": insecure
        }
    })
}

fn generate_wireguard_outbound(tag: &str, server: &str, port: u16, config: &Value) -> Value {
    let private_key = config.get("private_key").and_then(|v| v.as_str()).unwrap_or("");
    let public_key = config.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
    let address = config.get("address").and_then(|v| v.as_str()).unwrap_or("10.0.0.2/32");
    let dns = config.get("dns").and_then(|v| v.as_str()).unwrap_or("1.1.1.1");
    let mtu = config.get("mtu").and_then(|v| v.as_u64()).unwrap_or(1420) as u16;
    let allowed_ips = config.get("allowed_ips").and_then(|v| v.as_str()).unwrap_or("0.0.0.0/0");
    let persistent_keepalive = config.get("persistent_keepalive").and_then(|v| v.as_u64()).unwrap_or(25) as u16;

    json!({
        "type": "wireguard",
        "tag": tag,
        "server": server,
        "server_port": port,
        "private_key": private_key,
        "local_address": [address],
        "peer_public_key": public_key,
        "allowed_ips": [allowed_ips],
        "dns": [dns],
        "mtu": mtu,
        "persistent_keepalive_interval": persistent_keepalive
    })
}

fn generate_dns(node_tag: &str, dns_server: &str) -> Value {
    json!({
        "servers": [
            {
                "tag": "dns-remote",
                "address": format!("tls://{}", dns_server),
                "detour": node_tag
            },
            {
                "tag": "dns-direct",
                "address": dns_server,
                "detour": "direct",
                "strategy": "prefer_ipv4"
            },
            {
                "tag": "dns-block",
                "address": "rcode://success"
            }
        ],
        "rules": [
            {
                "outbound": ["any"],
                "server": "dns-block"
            },
            {
                "inbound": ["mixed-in", "tun-in"],
                "server": "dns-remote"
            }
        ],
        "final": "dns-remote",
        "strategy": "prefer_ipv4"
    })
}

fn generate_route_rules(rules: &[Value], node_tag: &str, bypass_ru: bool, bypass_local: bool, split_processes: &str) -> Vec<Value> {
    let mut route_rules: Vec<Value> = Vec::new();

    for rule in rules {
        let enabled = rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        if !enabled {
            continue;
        }

        let rule_type = rule.get("rule_type").and_then(|v| v.as_str()).unwrap_or("");
        let value = rule.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let node_id = rule.get("node_id").and_then(|v| v.as_str()).unwrap_or("direct");

        if value.is_empty() {
            continue;
        }

        let mut sing_rule = serde_json::Map::new();

        match rule_type {
            "domain_full" => {
                sing_rule.insert("domain".to_string(), json!([value]));
            }
            "domain_suffix" => {
                let domain = value.trim_start_matches("*.");
                sing_rule.insert("domain_suffix".to_string(), json!([domain]));
            }
            "domain_keyword" => {
                sing_rule.insert("domain_keyword".to_string(), json!([value]));
            }
            "domain_regex" => {
                sing_rule.insert("domain_regex".to_string(), json!([value]));
            }
            "ip_cidr" => {
                sing_rule.insert("ip_cidr".to_string(), json!([value]));
            }
            "process_name" => {
                sing_rule.insert("process_name".to_string(), json!([value]));
            }
            "geoip" => {
                sing_rule.insert("geoip".to_string(), json!([value]));
            }
            "geosite" => {
                sing_rule.insert("geosite".to_string(), json!([value]));
            }
            _ => {
                sing_rule.insert("domain".to_string(), json!([value]));
            }
        }

        sing_rule.insert("outbound".to_string(), json!(node_id));
        route_rules.push(Value::Object(sing_rule));
    }

    route_rules.push(json!({
        "domain_suffix": [
            "instagram.com", "cdninstagram.com", "threads.net",
            "facebook.com", "fbcdn.net", "fbsbx.com", "messenger.com",
            "whatsapp.com", "whatsapp.net", "oculus.com", "meta.com"
        ],
        "outbound": node_tag
    }));

    route_rules.push(json!({
        "domain_keyword": ["instagram", "cdninstagram", "facebook", "fbcdn", "whatsapp"],
        "outbound": node_tag
    }));

    // Built-in system rules
    route_rules.push(json!({
        "protocol": "dns",
        "outbound": "dns-out"
    }));

    route_rules.push(json!({
        "port": [53],
        "outbound": "dns-out"
    }));

    // Local network bypass
    if bypass_local {
        route_rules.push(json!({
            "ip_cidr": ["192.168.0.0/16", "10.0.0.0/8", "172.16.0.0/12", "127.0.0.0/8"],
            "outbound": "direct"
        }));
    }

    // Split tunneling - specific processes bypass VPN
    if !split_processes.is_empty() {
        let procs: Vec<&str> = split_processes.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if !procs.is_empty() {
            route_rules.push(json!({
                "process_name": procs,
                "outbound": "direct"
            }));
        }
    }

    // Russian services bypass - direct connection
    if bypass_ru {
        route_rules.push(json!({
            "domain_suffix": [
                "yandex.ru", "yandex.com", "ya.ru", "yandex.net",
                "vk.com", "vk.ru", "vkontakte.ru",
                "mail.ru", "icloud-mail.net",
                "rambler.ru", "lenta.ru", "rbc.ru",
                "ivi.ru", "ok.ru", "odnoklassniki.ru",
                "sberbank.ru", "tinkoff.ru", "alfabank.ru",
                "ozon.ru", "wildberries.ru", "lamoda.ru",
                "kinopoisk.ru", "tv.yandex.ru",
                "pikabu.ru", "dtf.ru", "vc.ru",
                "avito.ru", "drom.ru",
                "gosuslugi.ru", "mos.ru", "gov.ru",
                "rutube.ru", "smotrim.ru"
            ],
            "outbound": "direct"
        }));

        route_rules.push(json!({
            "domain_keyword": [
                "yandex", "mail.ru", "vk.com", "sberbank",
                "tinkoff", "ozon", "wildberries", "kinopoisk"
            ],
            "outbound": "direct"
        }));

        route_rules.push(json!({
            "geoip": ["ru"],
            "outbound": "direct"
        }));
    }

    route_rules
}

