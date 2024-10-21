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

extern crate gtk;

use gtk::cairo::{Antialias, Context, FontSlant, FontWeight};
use gtk::gdk::ffi::GdkRectangle;
use gtk::prelude::WidgetExt;

use crate::earth::FEET_PER_DEGREE;
use crate::model::airport::{Airport, Runway, Taxiway};
use crate::model::location::Location;

pub struct AirportPainter {
    // Define your Airport struct fields here
    pub draw_taxiways: bool,
    pub draw_runways: bool,
    pub draw_compass_rose: bool,
}

impl AirportPainter {
    pub fn draw_airport(&self, airport: &Airport, drawing_area: &gtk::DrawingArea, cr: &Context) {
        // Function to draw the airport on the GTK DrawingArea

        // Get the size of the DrawingArea
        let allocation = drawing_area.allocation();
        let width = allocation.width() as f64;
        let height = allocation.height() as f64;

        // Calculate the scale factor (pixels per foot)
        let extents = airport.calc_airport_extent();
        let height_feet = (extents[1] - extents[0]) * FEET_PER_DEGREE as f64;
        let width_feet = (extents[3] - extents[2])
            * FEET_PER_DEGREE as f64
            * airport.get_lat().to_radians().cos().abs();

        let scale_x = width_feet / width;
        let scale_y = height_feet / height;
        let scale = scale_x.max(scale_y);

        // Now calculate the actual width we will take at the scale to
        // calculate an offset to center the drawing
        let true_width = width_feet / scale;
        let true_height = height_feet / scale;
        // Rectangle to draw in is
        let offset_x = (width - true_width) / 2.0;
        let offset_y = (height - true_height) / 2.0;

        let bounding_box = GdkRectangle {
            x: offset_x as i32,
            y: offset_y as i32,
            width: true_width as i32,
            height: true_height as i32,
        };

        // Draw the runways

        // Draw the taxiways
        if self.draw_taxiways {
            let _ = cr.save();
            cr.set_source_rgb(0.7, 0.7, 0.7);
            for taxiway in airport
                .get_taxiways()
                .read()
                .expect("Could not get airport lock")
                .iter()
            {
                self.draw_taxiway(cr, &bounding_box, scale, &extents, taxiway, airport);
            }
            let _ = cr.restore();
        }

        if self.draw_runways {
            let _ = cr.save();
            cr.set_source_rgb(0.0, 0.0, 0.75);
            for runway in airport
                .get_runways()
                .read()
                .expect("Could not get airport lock")
                .iter()
            {
                self.draw_runway(cr, &bounding_box, scale, &extents, runway, airport);
            }
            self.draw_airport_name(cr, 0.0, 0.0, width, height, airport);
            let _ = cr.restore();
        }

        // Draw the compass rose
        if self.draw_compass_rose {
            let _ = cr.save();
            self.draw_compass_rose(cr, 0.0, 0.0, width, height);
            let _ = cr.restore();
        }
    }

    fn draw_runway(
        &self,
        cr: &Context,
        rectangle: &GdkRectangle,
        scale: f64,
        airport_extent: &[f64; 4],
        runway: &Runway,
        airport: &Airport,
    ) {
        let offset_lat = (runway.get_lat() - airport_extent[0]) * FEET_PER_DEGREE as f64;
        let offset_long = (runway.get_long() - airport_extent[2])
            * FEET_PER_DEGREE as f64
            * airport.get_lat().to_radians().cos();

        let heading_radians = runway.get_heading().to_radians();

        // Calculate the runway rectangle offset from airport origin
        // Corner offset component contributed by the runway length
        let feet_east_l = runway.get_length() as f64 / 2.0 * heading_radians.sin();
        let feet_north_l = runway.get_length() as f64 / 2.0 * heading_radians.cos();

        // Corner offset component contributed by the runway width
        let feet_east_w = runway.get_width() as f64 / 2.0 * heading_radians.cos();
        let feet_north_w = runway.get_width() as f64 / 2.0 * heading_radians.sin();

        // Get the corner offsets (Corners A,B,C,D)
        let a_lat = feet_north_l + feet_north_w + offset_lat;
        let a_long = feet_east_l - feet_east_w + offset_long;

        let b_lat = feet_north_l - feet_north_w + offset_lat;
        let b_long = feet_east_l + feet_east_w + offset_long;

        let c_lat = -feet_north_l - feet_north_w + offset_lat;
        let c_long = -feet_east_l + feet_east_w + offset_long;

        let d_lat = -feet_north_l + feet_north_w + offset_lat;
        let d_long = -feet_east_l - feet_east_w + offset_long;

        cr.move_to(
            a_long / scale + rectangle.x as f64,
            rectangle.height as f64 - a_lat / scale + rectangle.y as f64,
        );
        cr.line_to(
            b_long / scale + rectangle.x as f64,
            rectangle.height as f64 - b_lat / scale + rectangle.y as f64,
        );
        cr.line_to(
            c_long / scale + rectangle.x as f64,
            rectangle.height as f64 - c_lat / scale + rectangle.y as f64,
        );
        cr.line_to(
            d_long / scale + rectangle.x as f64,
            rectangle.height as f64 - d_lat / scale + rectangle.y as f64,
        );

        let _ = cr.fill();
    }

    fn draw_taxiway(
        &self,
        cr: &Context,
        rectangle: &GdkRectangle,
        scale: f64,
        airport_extent: &[f64; 4],
        taxiway: &Taxiway,
        airport: &Airport,
    ) {
        let mut first = true;
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for node in taxiway.get_nodes() {
            let offset_lat = (node.get_lat() - airport_extent[0]) * FEET_PER_DEGREE as f64;
            let offset_long = (node.get_long() - airport_extent[2])
                * FEET_PER_DEGREE as f64
                * airport.get_lat().to_radians().cos();

            let x = (offset_long / scale) + rectangle.x as f64;
            let y = rectangle.height as f64 - (offset_lat / scale) + rectangle.y as f64;
            if first {
                cr.move_to(x, y);
                prev_x = x;
                prev_y = y;
                first = false;
            } else {
                if node.get_type() == "112" || node.get_type() == "114" {
                    let offset_lat_b =
                        (node.get_bezier_lat() - airport_extent[0]) * FEET_PER_DEGREE as f64;
                    let offset_long_b = (node.get_bezier_long() - airport_extent[2])
                        * FEET_PER_DEGREE as f64
                        * (node.get_bezier_lat().to_radians().cos().abs());

                    let x_b = (offset_long_b / scale) + rectangle.x as f64;
                    let y_b = rectangle.height as f64 - (offset_lat_b / scale) + rectangle.y as f64;

                    cr.curve_to(
                        2.0 / 3.0 * x_b + 1.0 / 3.0 * prev_x,
                        2.0 / 3.0 * y_b + 1.0 / 3.0 * prev_y,
                        2.0 / 3.0 * x_b + 1.0 / 3.0 * x,
                        2.0 / 3.0 * y_b + 1.0 / 3.0 * y,
                        x,
                        y,
                    );
                    prev_x = x;
                    prev_y = y;
                } else {
                    cr.line_to(x, y);
                    prev_x = x;
                    prev_y = y;
                }

                if node.get_type() == "113" || node.get_type() == "114" {
                    cr.close_path();
                    first = true;
                }
            }
        }
        let _ = cr.fill();
    }

    fn draw_compass_rose(&self, cr: &Context, x: f64, y: f64, width: f64, _height: f64) {
        let north = "N";
        // Generate the text using a Serif font
        cr.select_font_face("Times", FontSlant::Normal, FontWeight::Bold);
        cr.set_font_size(16.0);

        // Use the text actual size to scale and position the arrow
        let text_extents = match cr.text_extents(north) {
            Ok(extents) => (extents.width(), extents.height()),
            _ => (5.0, 10.0),
        };

        let inc = text_extents.0 / 2.0;
        let offset_x = x + width - inc * 1.5;
        let offset_y = y + inc * 2.0 + 2.0;

        // Draw a little arrow
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_antialias(Antialias::Subpixel);
        cr.move_to(offset_x, offset_y - 2.0 * inc);
        cr.line_to(offset_x - inc, offset_y + 2.0 * inc);
        cr.line_to(offset_x, offset_y + inc);
        cr.line_to(offset_x + inc, offset_y + 2.0 * inc);
        let _ = cr.fill();
        cr.set_antialias(Antialias::None);

        if let Ok(extents) = cr.text_extents("N") {
            let t_w = extents.width();
            let t_h = extents.height();
            // cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.move_to(offset_x - t_w / 2.0, offset_y + 2.0 * inc + t_h + 2.0);
            let _ = cr.show_text(north);
            let _ = cr.stroke();
        }
    }

    fn draw_airport_name(
        &self,
        cr: &Context,
        x: f64,
        y: f64,
        _width: f64,
        _height: f64,
        airport: &Airport,
    ) {
        cr.select_font_face("Times", FontSlant::Normal, FontWeight::Bold);
        cr.set_font_size(14.0);
        let text = format!("{} - {}", airport.get_id(), airport.get_name());
        let text_extents = match cr.text_extents(&text) {
            Ok(extents) => (extents.width(), extents.height()),
            _ => (5.0, 10.0),
        };
        cr.move_to(x + 4.0, y + text_extents.1 + 2.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);

        let _ = cr.show_text(&text);
        let _ = cr.stroke();
    }

}
