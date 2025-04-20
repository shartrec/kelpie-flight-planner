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
#![forbid(unsafe_code)]

use gtk::FileFilter;
use gtk::gio::ListStore;
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use log::LevelFilter;
use std::error::Error;

pub(crate) mod airport_painter;
pub(crate) mod airport_parser;
pub(crate) mod distance_format;
pub(crate) mod fix_parser;
pub(crate) mod hour_format;
pub(crate) mod lat_long_format;
pub(crate) mod location_filter;
pub(crate) mod navaid_parser;
pub(crate) mod speed_format;
pub(crate) mod plan_writer_xml;
pub(crate) mod plan_reader;
pub(crate) mod plan_writer_route_manager;
pub(crate) mod fg_link;
pub(crate) mod info;

pub fn get_plan_file_filter(ext: &str) -> ListStore {
    let store = ListStore::new::<FileFilter>();
    let filter = FileFilter::new();
    filter.add_suffix(ext);
    store.append(&filter);
    let filter = FileFilter::new();
    filter.add_pattern("*");
    store.append(&filter);
    store
}

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self::init_logger();
        Logger
    }
    fn init_logger() {
        if let Some(home_path) = home::home_dir() {
            let log_path = home_path.join("kelpie-planner.log");
            let condition = RollingConditionBasic::new()
                .daily()
                .max_size(1024 * 1024);
            let file_appender =
                BasicRollingFileAppender::new(log_path, condition, 2);
            match file_appender {
                Ok(file) => {
                    let config = ConfigBuilder::new()
                        .set_time_offset_to_local()
                        .unwrap().build();
                    let config2 = ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .set_time_format_rfc3339()
                        .set_time_offset_to_local()
                        .unwrap().build();
                    CombinedLogger::init(vec![
                        TermLogger::new(
                            LevelFilter::Warn,
                            config,
                            TerminalMode::Mixed,
                            ColorChoice::Auto,
                        ),
                        WriteLogger::new(
                            LevelFilter::Info,
                            config2,
                            file,
                        ),
                    ]).unwrap_or_else(|e| {
                        Self::print_error(&e);
                    });
                    return;
                }
                Err(e) => {
                    Self::print_error(&e);
                }
            }
        }
        TermLogger::init(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ).unwrap_or_else(|e| {
            Self::print_error(&e);
        });
    }

    fn print_error(e: &dyn Error) {
        println!("Unable to initiate logger: {}", e);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        log::logger().flush();
    }
}