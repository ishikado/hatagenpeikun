/*
実際に bot が実行するコマンド
*/

use slack::{RtmClient};
use log::{warn, info};
// use slack::api::{Message};
// use slack::api::rtm::StartResponse;


pub fn on_echo(cli: &RtmClient, chid : &String, echo_arg : &String) -> Result<(), failure::Error> {
    info!("called on _echo : args ~ {}", echo_arg);
    if echo_arg.len() > 0 {
        let _ = cli.sender().send_message(chid, echo_arg);
    }
    else{
        warn!("echo_arg.len() == 0, so can't send echo message to slack");
    }
    return Ok(());
}

pub fn on_nowtime(_cli: &RtmClient, _chid : &String) -> Result<(), failure::Error> {
    return Ok(());
}
