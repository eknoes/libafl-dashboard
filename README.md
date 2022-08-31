# [WIP] Interactive LibAFL Dashboard
This project aims to create a dashboard for a fuzzing campaign for LibAFL.
Therefore, the OnDiskJSONMonitor must be used.
LibAFL Dashboard then serves a Dashboard via HTTP and continuously reads the logfile.
The data is sent via Websocket, so that the fuzzing server is loaded as little as possible.

The dashboard then shows current live statistics and is able to plot graphs for the different metrics.

## Screenshot
![Screenshot of the Interactive LibAFL Dashboard](resources/screenshot.png)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>