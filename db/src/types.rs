use diesel::*;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgTypeMetadata, PgMetadataLookup};
use diesel::types::{HasSqlType, NotNull};
use serde::Serialize;
use std::fmt::Display;

#[derive(SqlType)]
#[postgres(type_name = "regconfig")]
pub struct Regconfig;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Serialize)]
#[sql_type = "Regconfig"]
pub struct RegConfig(u32);

pub struct Tsvector;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Serialize)]
#[sql_type = "Tsvector"]
pub struct TsVector;

const ENGLISH: u32 = 13043;
const SPANISH: u32 = 13063;

impl Display for RegConfig {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            RegConfig(ENGLISH) => f.write_str("English"),
            RegConfig(SPANISH) => f.write_str("Spanish"),
            _ => f.write_str("Unsupported Language"),
        }
    }
}

impl FromSql<Regconfig, Pg> for RegConfig {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match u32::from_sql(bytes)? {
            oid => Ok(RegConfig(oid)),
        }
    }
}

impl HasSqlType<Tsvector> for Pg {
    fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 3614,
            array_oid: 3643,
        }
    }
}

impl NotNull for Tsvector {}

impl FromSql<Tsvector, Pg> for TsVector {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match u32::from_sql(bytes)? {
            _ => Ok(TsVector {}),
        }
    }
}