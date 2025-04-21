/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Flight Planner.
 *
 * Kelpie Flight Planner is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Flight Planner is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Flight Planner; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */

use std::process::Command;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "kelpie_planner.gresource",
    );

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()));
    let debug_dir = Path::new(&target_dir).join(profile);
    let dest_locale_dir = debug_dir.join("locale");
    println!("cargo:warning=dest locale={}", dest_locale_dir.display());

    let source_locale_dir = Path::new("resources/translations");
    println!("cargo:warning=source locale={}", source_locale_dir.display());

    // Compile .po files into .mo files
    compile_po_to_mo(source_locale_dir, &dest_locale_dir).expect("Failed to compile .po files");
}

fn compile_po_to_mo(base_dir: &Path, dest_dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(lang) = path.file_stem() {
            let lang = lang.to_string_lossy();
            let lc_messages_dir = dest_dir.join(&*lang).join("LC_MESSAGES");

            // Create the LC_MESSAGES directory if it doesn't exist
            fs::create_dir_all(&lc_messages_dir)?;

            if path.extension().map_or(false, |ext| ext == "po") {
                let mo_path = lc_messages_dir.join("kelpie_rust_planner.mo");

                // Compile using msgfmt
                let status = Command::new("msgfmt")
                    .arg(&path)
                    .arg("-o")
                    .arg(&mo_path)
                    .status()?;

                if !status.success() {
                    panic!("msgfmt failed on {}", path.display());
                }
            }
        }
    }
    Ok(())
}