use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use odbc::SqlTimestamp;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum SqlType {
    Mssql,
    Postgresql,
    Mysql,
    Generic,
}

pub fn to_dt(t: SqlTimestamp) -> NaiveDateTime {
    NaiveDate::from_ymd(t.year as i32, t.month as u32, t.day as u32).and_hms(
        t.hour as u32,
        t.minute as u32,
        t.second as u32,
    )
}
pub fn from_dt(dt: NaiveDateTime) -> SqlTimestamp {
    SqlTimestamp {
        year: dt.year() as i16,
        month: dt.month() as u16,
        day: dt.day() as u16,
        hour: dt.hour() as u16,
        minute: dt.minute() as u16,
        second: dt.second() as u16,
        fraction: 0,
    }
}

pub fn bool_to_sql(v: bool) -> i32 {
    if v {
        1
    } else {
        0
    }
}

pub struct SqlInsert<'a> {
    database: Option<&'a str>,
    table: &'a str,
    values: Vec<&'a str>,
}
impl<'a> SqlInsert<'a> {
    pub fn new(table: &'a str, values: Vec<&'a str>) -> Self {
        Self {
            database: None,
            table,
            values,
        }
    }
    pub fn new_with_db(database: &'a str, table: &'a str, values: Vec<&'a str>) -> Self {
        Self {
            database: Some(database),
            table,
            values,
        }
    }

    pub fn as_sql(self, sql_type: SqlType) -> String {
        let columns = {
            let mut buf = String::new();

            for i in 0..self.values.len() {
                let item = self.values[i];

                buf.push_str(&incase_name(item, sql_type));

                if i + 1 < self.values.len() {
                    buf.push_str(",");
                }
            }
            buf
        };
        let values = {
            let mut buf = String::with_capacity(self.values.len() * 2 - 1);
            for i in 0..self.values.len() {
                buf.push_str("?");
                if i + 1 < self.values.len() {
                    buf.push_str(",");
                }
            }
            buf
        };

        let sql = format!(
            "INSERT INTO {} ({}) VALUES({})",
            table_as_sql(&self.database, self.table, sql_type),
            columns,
            values
        );
        debug!("sql: {}", sql);

        sql
    }
}

pub struct SqlSelect<'a> {
    database: Option<&'a str>,
    table: &'a str,
    values: Vec<&'a str>,
    condition: &'a str,
}
impl<'a> SqlSelect<'a> {
    pub fn new(table: &'a str, values: Vec<&'a str>, condition: &'a str) -> Self {
        Self {
            database: None,
            table,
            values,
            condition,
        }
    }
    pub fn new_with_db(
        database: &'a str,
        table: &'a str,
        values: Vec<&'a str>,
        condition: &'a str,
    ) -> Self {
        Self {
            database: Some(database),
            table,
            values,
            condition,
        }
    }
    pub fn as_sql(self, sql_type: SqlType) -> String {
        let columns = {
            let mut buf = String::new();

            for i in 0..self.values.len() {
                let item = self.values[i];

                buf.push_str(&incase_name(item, sql_type));

                if i + 1 < self.values.len() {
                    buf.push_str(",");
                }
            }
            buf
        };

        let sql = format!(
            "SELECT {} FROM {} {};",
            columns,
            table_as_sql(&self.database, self.table, sql_type),
            self.condition
        );
        debug!("sql: {}", sql);

        sql
    }
}

pub struct SqlUpdate<'a> {
    database: Option<&'a str>,
    table: &'a str,
    columns: Vec<&'a str>,
    condition: &'a str,
}
impl<'a> SqlUpdate<'a> {
    pub fn new(table: &'a str, columns: Vec<&'a str>, condition: &'a str) -> Self {
        Self {
            database: None,
            table,
            columns,
            condition,
        }
    }
    pub fn new_with_db(
        database: &'a str,
        table: &'a str,
        columns: Vec<&'a str>,
        condition: &'a str,
    ) -> Self {
        Self {
            database: Some(database),
            table,
            columns,
            condition,
        }
    }

    pub fn as_sql(self, sql_type: SqlType) -> String {
        let mut columns = String::new();
        for column in self.columns {
            columns.push_str(&format!("{}=?", incase_name(column, sql_type)));
        }

        let sql = format!(
            "UPDATE {} SET {} {};",
            table_as_sql(&self.database, self.table, sql_type),
            columns,
            self.condition
        );
        debug!("sql: {}", sql);

        sql
    }
}

fn incase_name(name: &str, sql_type: SqlType) -> String {
    match sql_type {
        SqlType::Mssql => format!("[{}]", name),
        SqlType::Mysql => format!("`{}`", name),
        SqlType::Postgresql => format!("\"{}\"", name),
        SqlType::Generic => name.to_string(),
    }
}
fn incase_value(value: &str, sql_type: SqlType) -> String {
    match sql_type {
        SqlType::Mssql => format!("'{}'", value),
        SqlType::Mysql => format!("'{}'", value),
        SqlType::Postgresql => format!("'{}'", value),
        SqlType::Generic => format!("'{}'", value),
    }
}

fn table_as_sql<'a>(database: &Option<&'a str>, table: &'a str, sql_type: SqlType) -> String {
    if let Some(database) = database {
        if sql_type == SqlType::Mssql {
            format!(
                "{}.[dbo].{}",
                incase_name(database, sql_type),
                incase_name(table, sql_type)
            )
        } else {
            format!(
                "{}.{}",
                incase_name(database, sql_type),
                incase_name(table, sql_type)
            )
        }
    } else {
        incase_name(table, sql_type)
    }
}
