# Chameleon

A cross-platform desktop VPN client with Nuxt 3 frontend + Tauri v2 (Rust) backend + Sing-box core.

## Features

- **Protocols**: VLESS, VMess, Trojan, Shadowsocks, Hysteria2, WireGuard
- **Hybrid Routing**: Domain, IP, process-based rules with drag-and-drop priority
- **Node Management**: Import from subscription URLs, share links, or manual entry
- **Speed Testing**: TCP latency and download speed measurement per node
- **Auto-Switch**: Automatic node switching when latency exceeds threshold
- **Subscription Auto-Refresh**: Periodic subscription URL fetching
- **GeoIP/Geosite**: Country-based routing via MaxMind GeoLite2 + sing-box geo databases
- **System Tray**: Minimize to tray with connection status and quick node switching
- **Global Shortcuts**: Ctrl+Shift+V toggle connect, Ctrl+Shift+S show/hide, Ctrl+Shift+L toggle logs
- **Dark Glassy Dashboard**: Animated power button, real-time traffic stats, SpeedChart

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/) >= 1.77
- [Tauri CLI](https://v2.tauri.app/start/cli/): `cargo install tauri-cli --version "^2"`
- Platform-specific dependencies per [Tauri v2 docs](https://v2.tauri.app/start/prerequisites/)

## Quick Start

```bash
# Install frontend dependencies
cd nuxt-app && npm install && cd ..

# Run in development mode
npm run tauri:dev
```

The first build will automatically download:
- Sing-box v1.9.3 binary (`src-tauri/bin/`)
- GeoIP database (`src-tauri/geo/geoip.db`)
- Geosite database (`src-tauri/geo/geosite.db`)
- GeoLite2 Country database (`src-tauri/geo/GeoLite2-Country.mmdb`)

## Project Structure

```
├── nuxt-app/                 # Frontend (Nuxt 3 + TypeScript + TailwindCSS)
│   ├── pages/                # Route pages (dashboard, nodes, rules, settings)
│   ├── layouts/              # App layout with sidebar
│   ├── stores/               # Pinia stores (VPN state, nodes, rules)
│   ├── composables/          # Vue composables (useTauri, useVPN)
│   └── nuxt.config.ts        # Nuxt configuration
├── src-tauri/                # Backend (Rust + Tauri v2)
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── lib.rs            # App setup, 20+ Tauri commands, plugin init
│   │   ├── vpn_manager.rs    # Connection lifecycle, DB ops, auto-switch, subscription
│   │   ├── config_gen.rs     # Sing-box config generation for all protocols
│   │   ├── rules_engine.rs   # Domain/IP/GeoIP/Geosite rule evaluation
│   │   ├── speed_test.rs     # TCP latency test, parallel node testing
│   │   ├── geo_db.rs         # MaxMind GeoLite2 reader + geosite categories
│   │   └── tray_menu.rs      # System tray icon, context menu, shortcuts
│   ├── bin/                  # Sing-box binary (auto-downloaded)
│   ├── geo/                  # Geo databases (auto-downloaded)
│   ├── capabilities/         # Tauri v2 permission capabilities
│   └── tauri.conf.json       # Tauri configuration
└── package.json              # Root scripts (tauri:dev, tauri:build)
```

## Development

```bash
# Start development (hot-reload frontend + Tauri window)
npm run tauri:dev

# Build for production
npm run tauri:build

# Rust checks only
cd src-tauri && cargo check

# Frontend dev only (browser)
cd nuxt-app && npm run dev
```

The Nuxt dev server runs on `http://127.0.0.1:1420`. The Tauri CLI connects to it automatically.

## Build

```bash
npm run tauri:build
```

Output binaries:
- Windows: `src-tauri/target/release/vpn-client.exe` + MSI/NSIS installer
- macOS: `src-tauri/target/release/vpn-client` + DMG
- Linux: `src-tauri/target/release/vpn-client` + DEB/AppImage

## Configuration

Settings are stored in SQLite at `{app_data_dir}/client.db`:

| Key | Default | Description |
|-----|---------|-------------|
| `auto_switch` | `true` | Enable auto node switching |
| `latency_threshold_ms` | `2000` | Latency threshold for auto-switch |
| `subscription_url` | `""` | Subscription URL for auto-refresh |
| `subscription_interval_mins` | `60` | Subscription refresh interval |

## Tauri Commands

| Command | Description |
|---------|-------------|
| `vpn_connect` | Connect to a node |
| `vpn_disconnect` | Disconnect current VPN |
| `vpn_status` | Get current connection status |
| `vpn_switch_node` | Switch to a different node |
| `get_nodes` | List all nodes from database |
| `add_node` | Add a new node |
| `delete_node` | Delete a node |
| `test_node_speed` | Test latency and download speed |
| `test_all_nodes_speed` | Test all nodes in parallel |
| `import_subscription` | Import nodes from subscription URL |
| `get_rules` | List all routing rules |
| `add_rule` | Add a routing rule |
| `update_rule` | Update a routing rule |
| `delete_rule` | Delete a routing rule |
| `reorder_rules` | Reorder rules (drag-and-drop) |
| `test_rule` | Test domain against rules (supports GeoIP/Geosite) |
| `get_settings` | Get all settings |
| `update_settings` | Update settings |
| `export_logs` | Export connection logs to file |
