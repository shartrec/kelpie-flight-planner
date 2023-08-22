fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "kelpie_planner.gresource",
    );
}
