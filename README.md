# Interactive LibAFL Dashboard
This project aims to create a dashboard for a fuzzing campaign for LibAFL.
Therefore, the OnDiskJSONMonitor must be used.
LibAFL Dashboard then serves a Dashboard via HTTP and continuously reads the logfile.
The data is sent via Websocket, so that the fuzzing server is loaded as little as possible.

The dashboard then shows current live statistics and is able to plot graphs for the different metrics.

## Screenshot
![Screenshot of the Interactive LibAFL Dashboard](resources/screenshot.png)