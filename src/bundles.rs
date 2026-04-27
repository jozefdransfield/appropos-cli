use std::collections::HashMap;
use serde::Serialize;

pub fn list() -> Vec<Result<BundleVersion, Box<dyn std::error::Error>>> {
    platform::list()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::collections::HashMap;
    use crate::bundles::BundleVersion;
    use serde::Deserialize;
    use std::fs;

    #[derive(Deserialize)]
    struct InfoPlist {
        #[serde(rename = "CFBundleDisplayName")]
        display_name: Option<String>,
        #[serde(rename = "CFBundleName")]
        bundle_name: Option<String>,
        #[serde(rename = "CFBundleExecutable")]
        bundle_executable: Option<String>,
        #[serde(rename = "CFBundleIdentifier")]
        bundle_id: Option<String>,
        #[serde(rename = "CFBundleShortVersionString")]
        version: Option<String>,
        #[serde(rename = "SUFeedURL")]
        sparkle_url: Option<String>
    }

    pub fn list() -> Vec<Result<BundleVersion, Box<dyn std::error::Error>>> {
        let mut paths = Vec::new();

        scan_dir("/Applications", &mut paths, true);

        paths.sort();

        let mut results: Vec<Result<BundleVersion, Box<dyn std::error::Error>>> = Vec::new();

        for path in &paths {
            results.push(parse(path))
        }
        results
    }

    fn scan_dir(dir: &str, paths: &mut Vec<String>, read_nested: bool) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".app") {
                paths.push(entry.path().to_string_lossy().to_string());
            } else if entry.file_type().map(|t| t.is_dir() && read_nested).unwrap_or(false) {
                scan_dir(&entry.path().to_string_lossy(), paths, false);
            }
        }
    }

    pub(super) fn parse(path: &String) -> Result<BundleVersion, Box<dyn std::error::Error>> { //  e.to_string()
        let info: InfoPlist = plist::from_file(format!("{path}/Contents/Info.plist")).map_err(|e| format!("Failed to parse plist: {path} due to [{e}]"))?;

        let id = info.bundle_id.ok_or("missing CFBundleIdentifier")?;
        let version = info.version.ok_or("missing CFBundleShortVersionString")?;
        let sparkle_url = info.sparkle_url;

        let receipt = std::path::Path::new(&path).join("Contents/_MASReceipt/receipt");

        let source = String::from(if receipt.exists() { "appStore" } else if sparkle_url.is_some() { "sparkle" } else { "*" });

        let name = info.bundle_name.or(info.display_name).or(info.bundle_executable).ok_or("missing CFBundleName")?;

        let mut meta: HashMap<String, String> = HashMap::new();

        if sparkle_url.is_some() {
            meta.insert("sparkle_url".to_string(), sparkle_url.unwrap().to_string());
        }

        Ok(BundleVersion {
            name,
            id,
            version,
            source,
            meta
        })
    }
}

#[cfg(test)]
mod tests {
    use super::platform::parse;

    #[test]
    fn test_parse_plist() {
        let path = "/Applications/Adobe After Effects 2025/Adobe After Effects 2025.app/Contents/Info.plist".to_string();
        parse(&path).expect("failed to parse test plist");

        // let bundle = parse(&path).expect("failed to parse test plist");
        // assert_eq!(bundle.id, "com.example.testapp");
        // assert_eq!(bundle.name, "TestApp");
        // assert_eq!(bundle.version, "2.1.0");
        // assert_eq!(bundle.source, "sparkle");
        // assert_eq!(bundle.meta.get("sparkle_url").map(String::as_str), Some("https://example.com/appcast.xml"));
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    pub fn search() -> Vec<String> {
        eprintln!("apropos: unsupported platform");
        Vec::new()
    }
}

#[derive(Serialize)]
pub struct BundleVersion {
    pub name: String,
    pub id: String,
    pub version: String,
    pub source: String,
    pub meta: HashMap<String, String>
}
