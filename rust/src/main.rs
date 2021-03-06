extern crate ws;
extern crate clap;
extern crate serde;
extern crate serde_json;

use serde_json::Value;

use clap::{App, Arg};

const NULL_PAYLOAD: &'static Value = &Value::Null;

struct BenchHandler {
    ws: ws::Sender,
    count: u32,
}

impl ws::Handler for BenchHandler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Ok(body) = msg.as_text() {
            if let Ok(obj) = serde_json::from_str::<Value>(body) {
                if let Some((msg_type, payload)) = obj.as_object().map(|map| {
                    (map.get("type").and_then(Value::as_str).unwrap_or(""),
                     map.get("payload").unwrap_or(NULL_PAYLOAD))
                }) {
                    match msg_type {
                        "echo" => {
                            try!(self.ws.send(body));
                        }
                        "broadcast" => {
                            try!(self.ws.broadcast(body));
                            try!(self.ws.send(format!(r#"{{"type":"broadcastResult","listenCount": {}, "payload":{}}}"#,
                                                          self.count,
                                                          payload)))
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        self.count += 1;
        println!("Connection Open! {}", self.count);
        Ok(())
    }

    fn on_close(&mut self, _: ws::CloseCode, _: &str) {
        self.count -= 1;
        println!("Connection Closed! {}", self.count);
    }
}

fn main() {
    let matches = App::new("rust-ws-server")
                      .version("1.0")
                      .arg(Arg::with_name("address")
                               .short("a")
                               .long("address")
                               .required(true)
                               .takes_value(true)
                               .default_value("127.0.0.1"))
                      .arg(Arg::with_name("port")
                               .short("p")
                               .long("port")
                               .required(true)
                               .takes_value(true)
                               .default_value("3000"))
                      .get_matches();

    if let (Some(address), Some(port)) = (matches.value_of("address"), matches.value_of("port")) {
        ws::Builder::new()
            .with_settings(ws::Settings { max_connections: 500_000, ..Default::default() })
            .build(|out| {
                BenchHandler {
                    ws: out,
                    count: 0,
                }
            })
            .unwrap()
            .listen(&*format!("{}:{}", address, port))
            .unwrap();
    } else {
        println!("{}", matches.usage());
    }
}
