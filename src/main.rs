use clap::Parser;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::{thread};
use std::thread::sleep;
use std::time::Duration;
use tiny_http::{Header, Method, Response, Server};
use tungstenite::{accept, Error};
use tungstenite::Message::Text;

#[derive(Parser)]
#[clap(name = "LibAFL-Dashboard", version, author = "Sönke Huster", about = "Creates a webdashboard from a LibAFL Logfile")]
struct Cli {
    /// Hostname for the websocket
    #[clap(short, long, default_value = "0.0.0.0", required = false)]
    host: String,
    /// Hostname that is used to access the websocket from the browser
    #[clap(short, long, default_value = "localhost", required = false)]
    external_hostname: String,
    /// Port for the HTTP server
    #[clap(short, long, default_value = "9999", required = false)]
    port: usize,
    /// Port for the HTTP server
    #[clap(short, long, default_value = "9001", required = false)]
    websocket_port: usize,
    /// JSON-lines logfile from LibAFL
    #[clap(action)]
    logfile: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // Serve Webcontent
    let hostname = cli.host.clone();
    thread::Builder::new()
        .name("Webserver".into())
        .spawn(move || {
            let server = Server::http(format!("{}:{}", hostname, &cli.port)).unwrap();
            eprintln!("The server is now reachable at http://{}:{}", hostname, cli.port);
            for request in server.incoming_requests() {
                match *request.method() {
                    Method::Get => {
                        let response = match request.url() {
                            "/monitor.js" => Response::from_string(
                                include_str!("monitor.js")
                                    .replace("{{WSHOST}}", &cli.external_hostname)
                                    .replace("{{WSPORT}}", &cli.websocket_port.to_string()),
                            ).with_header(
                                Header::from_str(
                                    "Content-Type: Content-Type: text/javascript;charset=UTF-8",
                                )
                                    .unwrap(),
                            ),
                            "/plotly-basic.min.js" => Response::from_string(
                                include_str!("plotly-basic.min.js"),
                            ).with_header(
                                Header::from_str(
                                    "Content-Type: Content-Type: text/javascript;charset=UTF-8",
                                )
                                    .unwrap(),
                            ),
                            "/milligram.min.css" => Response::from_string(
                                include_str!("milligram.min.css"),
                            ).with_header(
                                Header::from_str(
                                    "Content-Type: Content-Type: text/css;charset=UTF-8",
                                )
                                .unwrap(),
                            ),
                            _ => Response::from_string(include_str!("index.html")).with_header(
                                Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap(),
                            ),
                        };
                        request.respond(response.with_status_code(200)).unwrap();

                        ()
                    }
                    _ => {}
                };
            }
        })
        .unwrap();

    // Websocket Things
    let server = TcpListener::bind(format!("{}:{}", cli.host, cli.websocket_port)).unwrap();
    for stream in server.incoming() {
        let logfile = cli.logfile.clone();
        thread::Builder::new()
            .name("Websocket Connection".into())
            .spawn(move || {
                println!("Accepted new Websocket connection");
                let mut websocket = accept(stream.unwrap()).unwrap();
                while !logfile.exists() {
                    println!("Logfile does not yet exist, waiting");
                    sleep(Duration::from_secs(5));
                }
                let file = File::open(&logfile).unwrap();
                let mut reader = BufReader::new(file);

                let (tx, rx) = channel();
                let mut watcher = watcher(tx, Duration::from_millis(5)).unwrap();
                watcher.watch(&logfile, RecursiveMode::Recursive).unwrap();

                loop {
                    let mut result = String::new();
                    match reader.read_line(&mut result) {
                        Ok(0) => {
                            loop {
                                match rx.recv() {
                                    Ok(DebouncedEvent::Write(_)) => {
                                        break;
                                    }
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("Error watching logfile: {}", e);
                                        return;
                                    }
                                }
                            }
                        }
                        Ok(_) => match websocket.write_message(Text(result)) {
                            Ok(_) => (),
                            Err(Error::Io(_)) => {
                                break;
                            },
                            Err(e) => {
                                eprintln!("Write Error: {:?}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    };
                }
                watcher.unwatch(&logfile).unwrap();
                println!("Thread stopped");
            })
            .unwrap();
    }
}
