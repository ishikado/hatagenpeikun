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
use log::{warn, info, debug};
use slack::api::{Message};
use slack::api::rtm::StartResponse;

#[derive(Debug, Fail)]
enum EventHandlerError {
    #[fail(display = "text_not_found")]
    TextNotFound,
    #[fail(display = "channel_not_found")]
    ChannelNotFound,
}

#[derive(Debug)]
pub struct MyHandler {
    start_response : Option<StartResponse>,
    myuid : String
}


impl MyHandler {
    pub fn new() -> MyHandler {
        return MyHandler{
            start_response : None,
            myuid : "".to_string()
        };
    }
    fn on_message(&mut self, cli: &RtmClient, message : &Message) -> Result<(), failure::Error> {
        match message {
            Message::Standard(ms) => {
                let bot_id = &ms.bot_id;
                // botのコメントには反応しない
                if *bot_id == None {
                    let text : &String = ms.text.as_ref().ok_or(EventHandlerError::TextNotFound)?;
                    let chid : &String = ms.channel.as_ref().ok_or(EventHandlerError::ChannelNotFound)?;
                    if let Some(pos) = text.find(self.myuid.as_str()) {
                        // 自分へのメンションに対する処理
                        // textから、メンション文字列を消す
                        let text_without_mention = &text[(pos+self.myuid.len()+1)..].trim_start().to_string();
                        // メンションに対する処理
                        self.on_mention(cli, chid, text_without_mention)?;
                        // メッセージ全般に対する処理
                        self.on_standard_message(cli, chid, message)?;
                    }
                }
            }
            _ => {
            }
        }
        return Ok(());
    }        

    fn on_standard_message(&mut self, _cli: &RtmClient, _chid : &String, _message : &Message) -> Result<(), failure::Error> {
        return Ok(());
    }

    fn on_mention(&mut self, cli: &RtmClient, chid : &String, text_without_mention : &String) -> Result<(), failure::Error> {
        use super::commands::*;

        // list of (help以外のcommandのdoc, command実行用クロージャ)
        let commands : Vec<(&str, Box<dyn Fn() -> Result<(), failure::Error>>)> = vec![
            ("echo <arg> - <arg> をそのまま返す",
             Box::new(move || {
                 let echo = "echo".to_string();
                 if let Some(_pos) = text_without_mention.find(echo.as_str()) {
                     let echo_arg = &text_without_mention[echo.len()..].trim_start().to_string();
                     on_echo(cli, chid, echo_arg)?;
                 }
                 return Ok(());
             })),
            ("nowtime - 現在時刻を取得する",
             Box::new(move || {
                 let nowtime = "nowtime".to_string();
                 if let Some(_pos) = text_without_mention.find(nowtime.as_str()) {
                     on_nowtime(cli, chid)?;
                 }
                 return Ok(());
             }))
        ];

        // helpだけは特別扱い
        {
            let docs = commands.iter().map(|(doc, _)| return *doc).collect::<Vec<&str>>();
            let helpdoc = "help - 使い方を表示する";
            let docs_with_help = [&docs[..], &vec![helpdoc]].concat();

            let help = "help".to_string();
            if let Some(_) = text_without_mention.find(help.as_str()) {
                on_help(cli, chid, docs_with_help)?;
            }
        }
        
        // help以外のcommandを実行
        // エラーが出たら終了
        for (_doc, f) in &commands {
            match f() {
                Ok(()) => {
                    continue;
                },
                Err(err) => {
                    return Err(err);
                }
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
