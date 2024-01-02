//! This module defines a struct and associated functions used for storing and SQlite database of shader-related paraphernalia.
//! 
//! # Version 1
//! There are two tables in version 1.
//! 
//! ## ShaderBytes
//! `ShaderBytes` holds the raw bytes for each shader, wihtout yakuza-specific wrappings. This is intended to be used when matching up shader names against 

use std::path::Path;

use rusqlite::{Connection, Transaction};

pub type DbResult<T> = rusqlite::Result<T>;

/// The version of the database the code expects to work with.
pub const CURR_DB_VERSION: u32 = 1;

pub struct ShaderDb {
    conn: Connection,
    version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}
impl ShaderStage {
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}
impl From<ShaderStage> for &'static str {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => "Vertex",
            ShaderStage::Fragment => "Fragment",
        }
    }
}
impl TryFrom<&str> for ShaderStage {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Vertex" => Ok(ShaderStage::Vertex),
            "Fragment" => Ok(ShaderStage::Fragment),
            _ => Err(format!("Invalid ShaderStage '{value}'"))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BytesType {
    DXBC,
}
impl BytesType {
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}
impl From<BytesType> for &'static str {
    fn from(value: BytesType) -> Self {
        match value {
            BytesType::DXBC => "DXBC",
        }
    }
}
impl TryFrom<&str> for BytesType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DXBC" => Ok(BytesType::DXBC),
            _ => Err(format!("Invalid BytesType '{value}'"))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisasmType {
    AMDIL,
}
impl DisasmType {
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}
impl From<DisasmType> for &'static str {
    fn from(value: DisasmType) -> Self {
        match value {
            DisasmType::AMDIL => "AMDIL",
        }
    }
}
impl TryFrom<&str> for DisasmType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AMDIL" => Ok(DisasmType::AMDIL),
            _ => Err(format!("Invalid DisasmType '{value}'"))
        }
    }
}

impl ShaderDb {
    pub fn from_file<P: AsRef<Path>>(path: P) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let version = conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| row.get(0))?;
        if version > CURR_DB_VERSION {
            panic!("Database loaded with higher user_version than supported by this binary. Please update this tool to the latest version.");
        }
        let mut db = Self {
            conn, version
        };
        db.migrate()?;
        Ok(db)
    }

    /// Perform a migration and update the DB internal user_version all in one transaction
    fn push_version<F: FnOnce(&Transaction) -> DbResult<()>>(&mut self, f: F, new_version: u32) -> DbResult<()> {
        let tx = self.conn.transaction()?;
        f(&tx)?;
        tx.pragma_update(None, "user_version", new_version)?;
        tx.commit()?;
        self.version = new_version;
        Ok(())
    }

    /// Perform all necessary version migrations from the current version (self.version) all the way up to CURR_DB_VERSION
    fn migrate(&mut self) -> DbResult<()> {
        if self.version == 0 {
            self.push_version(|tx| {
                tx.execute("CREATE TABLE ShaderBytes (
                    Category TEXT NOT NULL,
                    ShaderName TEXT NOT NULL,
                    ShaderStage TEXT NOT NULL,
                    BytesType TEXT NOT NULL,
                    Bytes BLOB NOT NULL,
                    SHA256 BLOB NOT NULL
                )", [])?;
                tx.execute("CREATE TABLE ShaderDisasm (
                    Category TEXT NOT NULL,
                    ShaderName TEXT NOT NULL,
                    ShaderStage TEXT NOT NULL,
                    DisasmType TEXT NOT NULL,
                    Disasm TEXT NOT NULL
                )", [])?;
                Ok(())
            }, 1)?;
        }
        // Insert further migrations here when necessary.
        assert_eq!(self.version, CURR_DB_VERSION);
        Ok(())
    }

    pub fn insert_bytes(&mut self, category: &str, shader_name: &str, shader_stage: ShaderStage, bytes_type: BytesType, bytes: &[u8]) -> DbResult<()> {
        use ring::digest::{Context, SHA256};
        let mut ctx = Context::new(&SHA256);
        ctx.update(bytes);
        let digest = ctx.finish();
        self.conn.execute(
            "INSERT INTO ShaderBytes (Category, ShaderName, ShaderStage, BytesType, Bytes, SHA256) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (category, shader_name, shader_stage.to_str(), bytes_type.to_str(), bytes, digest.as_ref())
        )?;
        Ok(())
    }

    pub fn insert_disasm(&mut self, category: &str, shader_name: &str, shader_stage: ShaderStage, disasm_type: DisasmType, disasm: &str) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO ShaderDisasm (Category, ShaderName, ShaderStage, DisasmType, Disasm) VALUES (?1, ?2, ?3, ?4, ?5)",
            (category, shader_name, shader_stage.to_str(), disasm_type.to_str(), disasm)
        )?;
        Ok(())
    }
}
