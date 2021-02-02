use mysql::prelude::*;

fn env_lossy(key: &str) -> Option<String> {
    std::env::var_os(key).map(|s| s.to_string_lossy().into_owned())
}

fn mysql_connect() -> Result<mysql::Conn, Box<dyn std::error::Error>> {
    let opts = mysql::OptsBuilder::new()
        .ip_or_hostname(env_lossy("ZM_DB_HOST"))
        .db_name(env_lossy("ZM_DB_NAME"))
        .user(env_lossy("ZM_DB_USER"))
        .pass(env_lossy("ZM_DB_PASS"));
    Ok(mysql::Conn::new(opts)?)
}

#[derive(Debug, Clone, Copy)]
pub struct MonitorSpec {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub colors: u32,
}

impl FromRow for MonitorSpec {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError> {
        let (id, width, height, colors) = <(u32, u32, u32, u32)>::from_row_opt(row)?;
        Ok(Self { id, width, height, colors })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut conn = mysql_connect()?;
    println!("{:#?}", conn.query::<MonitorSpec, _>("select Id, Width, Height, Colours from Monitors")?);
    Ok(())
}
