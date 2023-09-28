use crate::earth::coordinate::Coordinate;

pub trait Location: {
    fn get_elevation(&self) -> &i32;
    fn get_id(&self) -> &str;
    fn get_lat(&self) -> &f64;
    fn get_lat_as_string(&self) -> String;
    fn get_long(&self) -> &f64;
    fn get_long_as_string(&self) -> String;
    fn get_loc(&self) -> &Coordinate;
    fn get_name(&self) -> &str;
}
