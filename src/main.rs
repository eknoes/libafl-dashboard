use clap::Parser;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tiny_http::{Header, Method, Response, Server};
use tungstenite::accept;
use tungstenite::Message::Text;

#[derive(Parser)]
#[clap(name = "LibAFL-Dashboard", version, author = "Sönke Huster")]
struct Cli {
    #[clap(short, long, default_value = "0.0.0.0", required = false)]
    host: String,
    #[clap(short, long, default_value = "localhost", required = false)]
    external_hostname: String,
    #[clap(short, long, default_value = "9999", required = false)]
    port: usize,
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

            for request in server.incoming_requests() {
                match *request.method() {
                    Method::Get => {
                        let response = match request.url() {
                            "/monitor.js" => Response::from_string(
                                include_str!("monitor.js")
                                    .replace("{{WSHOST}}", &cli.external_hostname),
                            )
                            .with_header(
                                Header::from_str(
                                    "Content-Type: Content-Type: text/javascript;charset=UTF-8",
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
    let server = TcpListener::bind(format!("{}:{}", cli.host, 9001)).unwrap();
    for stream in server.incoming() {
        let logfile = cli.logfile.clone();
        thread::Builder::new()
            .name("Websocket Connection".into())
            .spawn(move || {
                println!("Accepted new Websocket connection");
                let mut websocket = accept(stream.unwrap()).unwrap();
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