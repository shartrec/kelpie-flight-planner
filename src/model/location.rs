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

use crate::earth::coordinate::Coordinate;

pub trait Location {
    fn get_elevation(&self) -> &i32;
    fn get_id(&self) -> &str;
    fn get_lat(&self) -> &f64;
    fn get_lat_as_string(&self) -> String;
    fn get_long(&self) -> &f64;
    fn get_long_as_string(&self) -> String;
    fn get_loc(&self) -> &Coordinate;
    fn get_name(&self) -> &str;
}
