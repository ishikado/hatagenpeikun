//
// Copyright 2014-2016 the slack-rs authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//
// This is a simple example of using slack-rs.
// You can run it with `cargo run <api_key>`
//
// NOTE: This will post in the #general channel of the account you connect
// to.
//

use getopts::Options;
use hatagenpeikun::event_handler::MyHandler;
use hatagenpeikun::hatagenpei::controller::DataStore;
use log::error;
use slack::RtmClient;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage : {} SLACK_API_TOKEN [options]", program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut opts = Options::new();
    opts.optopt(
        "l",
        "loglevel",
        "set loglevel",
        "debug | info | warn | error",
    );

    opts.optopt("r", "redis_uri", "set redis uri", "");
    opts.optopt("p", "postgre_uri", "set postgre uri", "");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    // help
    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        return;
    }

    let loglevel: String = match matches.opt_str("l") {
        Some(level) => level,
        _ => "info".to_string(), // default log level
    };

    let api_key = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&args[0], opts);
        return;
    };

    env::set_var("RUST_LOG", loglevel);
    env_logger::init();

    // url が 複数指定されていた場合、使用する datastore は redis が優先される
    let mut handler = match matches.opt_str("r") {
        Some(uri) => MyHandler::new(DataStore::Redis { uri: uri }),
        _ => match matches.opt_str("p") {
            Some(uri) => MyHandler::new(DataStore::Postgre { uri: uri }),
            _ => MyHandler::new(DataStore::OnMemory),
        },
    };

    let r = RtmClient::login_and_run(&api_key, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => {
            error!("Error: {}", err);
            return;
        }
    }
}
