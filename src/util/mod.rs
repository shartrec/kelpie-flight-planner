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

use gtk::FileFilter;
use gtk::gio::ListStore;
use gtk::prelude::StaticType;

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

pub fn get_plan_file_filter(ext: &str) -> ListStore {

    let store = ListStore::new(FileFilter::static_type());
    let filter = FileFilter::new();
    filter.add_suffix(ext);
    store.append(&filter);
    let filter = FileFilter::new();
    filter.add_pattern("*");
    store.append(&filter);
    store
}