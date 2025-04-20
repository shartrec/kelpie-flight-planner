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

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_locale_dir = out_dir.join("locale");
    println!("dest locale={}", dest_locale_dir.display());

    let source_locale_dir = Path::new("locale");
    println!("source locale={}", source_locale_dir.display());

    // Compile .po files into .mo files
    compile_po_to_mo(source_locale_dir).expect("Failed to compile .po files");

    // Make OUT_DIR/locale path available at runtime
    println!("cargo:rustc-env=LOCALE_DIR={}", dest_locale_dir.display());

}

fn compile_po_to_mo(base_dir: &Path) -> std::io::Result<()> {
    for lang_dir in fs::read_dir(base_dir)? {
        let lang_dir = lang_dir?;
        let lc_messages = lang_dir.path().join("LC_MESSAGES");

        if lc_messages.exists() {
            for entry in fs::read_dir(&lc_messages)? {
                let entry = entry?;
                let path = entry.path();

                if let Some(ext) = path.extension() {
                    if ext == "po" {
                        let mo_path = path.with_extension("mo");

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
        }
    }
    Ok(())
}
