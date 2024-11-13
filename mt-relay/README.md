# MultiTherm serial-to-npz relay

Converts an incoming stream of data from a MultiTherm connected over USB serial to a NumPy Zip ([npyz](https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html)).

Bill of Materials:
- [serialport](https://github.com/serialport/serialport-rs) = "4.6.0"
- [nalgebra](https://docs.rs/nalgebra/latest/nalgebra/) = "0.33.2"
- [npyz](https://github.com/ExpHP/npyz) = "0.8"
