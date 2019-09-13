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
use log::{error, warn, info, debug};
use slack::api::{Message, MessageStandard};
use slack::api::rtm::StartResponse;

#[derive(Debug, Fail)]
enum EventHandlerError {
    #[fail(display = "text_not_found")]
    TextNotFound,
    #[fail(display = "channel_not_found")]
    ChannelNotFound
}

#[derive(Debug)]
pub struct MyHandler {
    start_response : Option<StartResponse>,
    myuid : String
}

impl MyHandler {
    pub fn new() -> MyHandler {
        return MyHandler{start_response : None,
                         myuid : "".to_string()
        };
    }
    fn on_message(&mut self, cli: &RtmClient, message : &Message) -> Result<(), failure::Error> {
        match message {
            Message::Standard(ms) => {
                // 自分へのメンションに対する処理
                {
                    let text : &String = ms.text.as_ref().ok_or(EventHandlerError::TextNotFound)?;
                    let chid : &String = ms.channel.as_ref().ok_or(EventHandlerError::ChannelNotFound)?;
                    let bot_id = &ms.bot_id;
                    // botのmentionには反応しない
                    if *bot_id == None {
                        if text.find(self.myuid.as_str()) != None {
                            // TODO : textから、メンション文字列を消す
                            let _ = cli.sender().send_message(chid, text);
                        }
                    }
                    // debug!("message.txt = {:?}", ms.text);
                }
            }
            _ => {
            }
        }
        return Ok(());
    }        
}

#[allow(unused_variables)]
impl slack::EventHandler for MyHandler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        debug!("on_event(event: {:?})", event);
        match event {
            Event::Hello => {
            },
            Event::Message(m) => {
                match self.on_message(cli, &(*m)) {
                    Ok(()) => {}
                    Err(err) => {
                        warn!("Error occured ! = {:?}", err);
                    }
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
        let uid = cli.start_response()
            .slf
            .as_ref()
            .and_then(|user| {
                user.id.as_ref()
            }).expect("user.id is not found").clone();

        self.start_response = Some(cli.start_response().clone());
        self.myuid = uid;
        // Send a message over the real time api websocket
    }
}
