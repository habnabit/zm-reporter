pub(crate) fn env_lossy(key: &str) -> Option<String> {
    std::env::var_os(key).map(|s| s.to_string_lossy().into_owned())
}

pub fn load_env() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_path("/etc/zm/zm.conf")?;
    let mut paths = std::fs::read_dir("/etc/zm/conf.d")?
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    match path.extension() {
                        Some(ext) if ext == "conf" => Some(Ok(path)),
                        _ => None,
                    }
                },
                Err(e) => Some(Err(e)),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for p in paths {
        dotenv::from_path(p)?;
    }
    Ok(())
}
