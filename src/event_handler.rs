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
//


use slack::{Event, RtmClient};
use std::env;
use log::{error, warn, info, debug};
use slack::api::{Message, MessageStandard};

pub struct MyHandler {
    test_channel_id : String
}

impl MyHandler {
    pub fn new() -> MyHandler {
        return MyHandler{test_channel_id : "".to_string()};
    }
}


#[allow(unused_variables)]
impl slack::EventHandler for MyHandler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        debug!("on_event(event: {:?})", event);
        match event {
            Event::Hello => {
                // // hello を受け取ったら、hello worldを #general に投稿する
                // let general_channel_id = &self.test_channel_id;
                // let _ = cli.sender().send_message(&general_channel_id, "Hello world! (rtm)");
            },
            Event::Message(m) => {
                match *m {
                    Message::Standard(ms) => {
                        
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn on_close(&mut self, cli: &RtmClient) {
        info!("on_close");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        info!("on_connect");
        // find the general channel id from the `StartResponse`
        let general_channel_id = cli.start_response()
            .channels
            .as_ref()
            .and_then(|channels| {
                channels
                    .iter()
                    .find(|chan| match chan.name {
                        None => false,
                        Some(ref name) => name == "general",
                    })
            })
            .and_then(|chan| chan.id.as_ref())
            .expect("general channel not found");
        self.test_channel_id = general_channel_id.clone();
        let _ = cli.sender().send_message(&general_channel_id, "Hello world! (rtm)");
        // Send a message over the real time api websocket
    }
}
