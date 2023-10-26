# kelpie-flight-planner
Kelpie flight planner for FlightGear - the Rust veresion

This Gui flight planner uses the airport and navaid databases of the Flightgear, allowing the user to search for airports and navigation aids and plan flights between them.

This project is a still very much a work in progress. An older fully featured and fuctioning JAv based projec is available at https://sourceforge.net/projects/fgflightplanner/

Currently all development is taking place on Linux and this is the only currently targetted platform.

To install and test, please clone the repository and then build using ```cargo build``` and test using either ```cargo run``` or go to the target directory and run the executable.

*Before* running you need to convert the apt.dat, nav.dat and fix.dat files to unicode.  Try 
```
iconv -f ISO-8859-15 -t UTF-8 fix.dat > fix2.dat
```
and store them directory.  

You also need to copy the preferences file to the ~/.config/kelpie-flight-planner and edit it as required.

The world shoreline also needs to be downloaded.  ***Work in Progress***
