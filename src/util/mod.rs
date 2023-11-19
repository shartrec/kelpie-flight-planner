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