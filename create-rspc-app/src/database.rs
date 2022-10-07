use std::{fs::create_dir_all, io, path::Path, str::FromStr};

use include_dir::{include_dir, Dir};
use strum::EnumIter;

use crate::{framework::Framework, utils::replace_in_file};

static AXUM_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/axum_pcr_base");
static TAURI_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/tauri_pcr_base");

#[derive(Debug, EnumIter, PartialEq, Eq)]
pub enum Database {
    PrismaClientRust,
    None,
}

impl ToString for Database {
    fn to_string(&self) -> String {
        match self {
            Self::PrismaClientRust => "Prisma Client Rust",
            Self::None => "None",
        }
        .to_string()
    }
}

impl FromStr for Database {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Prisma Client Rust" => Ok(Self::PrismaClientRust),
            "None" => Ok(Self::None),
            _ => Err(format!("{} is not a valid database", s)),
        }
    }
}

impl Database {
    pub fn render(&self, path: &Path, project_name: &str, framework: Framework) -> io::Result<()> {
        match framework {
            Framework::Tauri => {
                create_dir_all(&path).unwrap();
                TAURI_BASE_TEMPLATE.extract(path)?;

                replace_in_file(
                    path.join("src-tauri").join("Cargo.toml").as_path(),
                    "{{name}}",
                    project_name,
                )?;
            }
            Framework::Axum => {
                create_dir_all(&path).unwrap();
                AXUM_BASE_TEMPLATE.extract(path)?;

                replace_in_file(
                    path.join("src-tauri").join("Cargo.toml").as_path(),
                    "{{name}}",
                    project_name,
                )?;
            }
        }

        Ok(())
    }
}
