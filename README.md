# kelpie-flight-planner
**Kelpie flight planner for FlightGear v2.0 - the Rust version**

This Gui flight planner uses the airport and navaid databases of the Flightgear, allowing the user to search for airports and navigation aids and plan flights between them.

Currently, all development is taking place on Linux and this is the best supported platform.

Only Linux binaries are provided in this release.

You should be able to build this project on other platforms by following the instructions in the book
[GUI development with Rust and GTK 4,](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)  

To install and test, please clone the repository and then build using ```cargo build --release``` and test
using either ```cargo run --release``` or go to the target directory and run the executable.

Major changes in this release include:
- The project has been rewritten in Rust, using the GTK 4 library for the GUI.
- OpenGL rendering has been modernised to GLES 3.0, which should give better performance and visual quality.
- The World Map now uses NASA's Blue Marble Next Generation (BMNG) dataset, which provides an image of the Earth's surface, rather than drawing shorelines.
- The planner now uses Dijkstra's algorithm for pathfinding, which gives a generally better plan than the original naive algorithm.
