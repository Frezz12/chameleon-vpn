use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn download_with_redirects(url: &str, dest: &PathBuf) -> bool {
    let status = Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap(), url])
        .status();
    if let Ok(s) = status {
        if s.success() && dest.exists() {
            return true;
        }
    }
    false
}

#[cfg(unix)]
fn set_unix_executable(path: &PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn set_unix_executable(_path: &PathBuf) {}
fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let version = "1.9.3";

    // ======== 1. Prepare directories ========
    let bin_dir = manifest_dir.join("bin");
    fs::create_dir_all(&bin_dir).ok();

    let geo_dir = manifest_dir.join("geo");
    fs::create_dir_all(&geo_dir).ok();

    // ======== 2. Download sing-box binary ========
    let ext = if target_os == "windows" { "zip" } else { "tar.gz" };
    let binary_name = if target_os == "windows" { "sing-box.exe" } else { "sing-box" };
    let binary_path = bin_dir.join(binary_name);

    if !binary_path.exists() {
        let arch = match target_arch.as_str() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            "x86" => "386",
            other => other,
        };

        let os = match target_os.as_str() {
            "windows" => "windows",
            "macos" => "darwin",
            "linux" => "linux",
            other => other,
        };

        let url = format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{version}/sing-box-{version}-{os}-{arch}.{ext}"
        );

        println!("cargo:warning=Downloading sing-box from: {url}");

        let archive = bin_dir.join(format!("sing-box-{version}-{os}-{arch}.{ext}"));

        if download_with_redirects(&url, &archive) {
            println!("cargo:warning=sing-box archive downloaded");

            // Extract
            if ext == "zip" {
                Command::new("powershell")
                    .args([
                        "-Command",
                        &format!(
                            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                            archive.display(),
                            bin_dir.display()
                        ),
                    ])
                    .status()
                    .ok();
                let _ = fs::rename(
                    bin_dir.join(format!("sing-box-{version}-{os}-{arch}")).join("sing-box.exe"),
                    &binary_path,
                );
            } else {
                Command::new("tar")
                    .args(["-xzf", archive.to_str().unwrap(), "-C", bin_dir.to_str().unwrap()])
                    .status()
                    .ok();
                let _ = fs::rename(
                    bin_dir.join(format!("sing-box-{version}-{os}-{arch}")).join("sing-box"),
                    &binary_path,
                );
            }

            if binary_path.exists() {
                set_unix_executable(&binary_path);
                println!("cargo:warning=sing-box downloaded successfully to {:?}", binary_path);
            } else {
                println!("cargo:warning=Failed to extract sing-box binary");
            }
        } else {
            println!("cargo:warning=Failed to download sing-box. Place the binary at: {:?}", binary_path);
        }
    } else {
        println!("cargo:warning=sing-box already exists at {:?}, skipping download", binary_path);
    }

    // ======== 3. Download GeoIP / Geosite databases ========

    let geoip_path = geo_dir.join("geoip.db");
    if !geoip_path.exists() || geoip_path.metadata().map(|m| m.len()).unwrap_or(0) < 100_000 {
        let url = "https://github.com/SagerNet/sing-geoip/releases/latest/download/geoip.db".to_string();
        println!("cargo:warning=Downloading geoip.db from: {url}");
        let _ = fs::remove_file(&geoip_path);
        if download_with_redirects(&url, &geoip_path) {
            let size = geoip_path.metadata().map(|m| m.len()).unwrap_or(0);
            println!("cargo:warning=geoip.db downloaded successfully ({} bytes)", size);
        } else {
            println!("cargo:warning=Failed to download geoip.db. Sing-box geoip routing may not work.");
        }
    } else {
        println!("cargo:warning=geoip.db already exists ({} bytes), skipping download", geoip_path.metadata().map(|m| m.len()).unwrap_or(0));
    }

    let geosite_path = geo_dir.join("geosite.db");
    if !geosite_path.exists() || geosite_path.metadata().map(|m| m.len()).unwrap_or(0) < 100_000 {
        let url = "https://github.com/SagerNet/sing-geosite/releases/latest/download/geosite.db".to_string();
        println!("cargo:warning=Downloading geosite.db from: {url}");
        let _ = fs::remove_file(&geosite_path);
        if download_with_redirects(&url, &geosite_path) {
            let size = geosite_path.metadata().map(|m| m.len()).unwrap_or(0);
            println!("cargo:warning=geosite.db downloaded successfully ({} bytes)", size);
        } else {
            println!("cargo:warning=Failed to download geosite.db. Sing-box geosite routing may not work.");
        }
    } else {
        println!("cargo:warning=geosite.db already exists ({} bytes), skipping download", geosite_path.metadata().map(|m| m.len()).unwrap_or(0));
    }

    let mmdb_path = geo_dir.join("GeoLite2-Country.mmdb");
    let mmdb_valid = mmdb_path.exists()
        && mmdb_path.metadata().map(|m| m.len()).unwrap_or(0) > 1_000_000
        && std::fs::read(&mmdb_path)
            .ok()
            .and_then(|d| d.get(..4).map(|h| h == b"\xab\xcd\x12\xef"))
            .unwrap_or(false);
    if !mmdb_valid {
        let _ = fs::remove_file(&mmdb_path);
        let urls = [
            "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-Country.mmdb",
            "https://github.com/Loyalsoldier/geoip/releases/latest/download/Country.mmdb",
        ];
        let mut downloaded = false;
        for url in &urls {
            println!("cargo:warning=Downloading GeoLite2-Country.mmdb from: {url}");
            if download_with_redirects(url, &mmdb_path) {
                let size = mmdb_path.metadata().map(|m| m.len()).unwrap_or(0);
                let valid = std::fs::read(&mmdb_path)
                    .ok()
                    .and_then(|d| d.get(..4).map(|h| h == b"\xab\xcd\x12\xef"))
                    .unwrap_or(false);
                if valid {
                    println!("cargo:warning=GeoLite2-Country.mmdb downloaded successfully ({} bytes)", size);
                    downloaded = true;
                    break;
                } else {
                    println!("cargo:warning=GeoLite2-Country.mmdb from {url} has invalid format ({} bytes), trying next...", size);
                    let _ = fs::remove_file(&mmdb_path);
                }
            } else {
                println!("cargo:warning=Failed to download from {url}, trying next...");
            }
        }
        if !downloaded {
            println!("cargo:warning=Could not download valid GeoLite2-Country.mmdb. Rust-side geoip lookups disabled.");
        }
    } else {
        println!("cargo:warning=GeoLite2-Country.mmdb already exists and valid, skipping download");
    }

    // ======== 4. Tauri build (now all resources exist) ========
    tauri_build::build();
}
