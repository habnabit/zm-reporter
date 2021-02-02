use mysql::prelude::*;
use std::collections::BTreeMap;

use crate::env::env_lossy;

pub fn mysql_connect() -> Result<mysql::Conn, Box<dyn std::error::Error>> {
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
    pub width: usize,
    pub height: usize,
    pub colors: u32,
    pub buffer_image_count: usize,
}

#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub name: String,
    pub status: String,
}

pub fn monitor_specs_from_mysql(
    conn: &mut mysql::Conn,
) -> Result<BTreeMap<u32, (MonitorSpec, MonitorStatus)>, Box<dyn std::error::Error>> {
    let rows: Vec<(u32, usize, usize, u32, usize, String, String)> = conn.query(
        r"
        select Id, Width, Height, Colours, ImageBufferCount, Name, Status
        from Monitors
        left join Monitor_Status on Id = MonitorId
        where Enabled
    ",
    )?;
    let ret = rows
        .into_iter()
        .map(|(id, width, height, colors, buffer_image_count, name, status)| {
            (
                id,
                (
                    MonitorSpec {
                        id,
                        width,
                        height,
                        colors,
                        buffer_image_count,
                    },
                    MonitorStatus { name, status },
                ),
            )
        })
        .collect();
    Ok(ret)
}
