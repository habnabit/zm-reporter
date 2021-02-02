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
}

impl FromRow for MonitorSpec {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError> {
        let (id, width, height, colors) = <(u32, usize, usize, u32)>::from_row_opt(row)?;
        Ok(Self { id, width, height, colors })
    }
}

#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub status: String,
}

pub fn monitor_specs_from_mysql(conn: &mut mysql::Conn) -> Result<BTreeMap::<u32, (MonitorSpec, MonitorStatus)>, Box<dyn std::error::Error>> {
    let rows: Vec<(u32, usize, usize, u32, String)> = conn.query(r"
        select Id, Width, Height, Colours, Status
        from Monitors
        left join Monitor_Status on Id = MonitorId
        where Enabled
    ")?;
    let ret = rows.into_iter()
        .map(|(id, width, height, colors, status)| (id, (MonitorSpec { id, width, height, colors }, MonitorStatus { status })))
        .collect();
    Ok(ret)
}
