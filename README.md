# kelpie-flight-planner
Kelpie flight planner for FlightGear v2.0 - the Rust version

This Gui flight planner uses the airport and navaid databases of the Flightgear, allowing the user to search for airports and navigation aids and plan flights between them.

This project is a still a work in progress. An older fully featured and 
fuctioning Java based project is available at https://sourceforge.net/projects/fgflightplanner/

Currently all development is taking place on Linux and this is the best supported platform.

Both Windows and Linux binaries are provided in the alpha release.

You should be able to build this project on other platforms by following the instructions in the book
[GUI development with Rust and GTK 4,](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)  

To install and test, please clone the repository and then build using ```cargo build --release``` and test
using either ```cargo run --release``` or go to the target directory and run the executable.
