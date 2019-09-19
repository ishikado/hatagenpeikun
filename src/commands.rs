/*
実際に bot が実行するコマンド
*/

use slack::{RtmClient};
use log::{warn, info};


/*
public functions
*/

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

// 現在時刻を取得し、chid で指定されたチャンネルに投稿する
pub fn on_nowtime(cli: &RtmClient, chid : &String) -> Result<(), failure::Error> {
    let nowtimestr = get_nowtime_string();
    let _ = cli.sender().send_message(chid, &nowtimestr);
    return Ok(());
}


/*
private functions
*/

fn get_nowtime_string() -> String {
    use chrono::{Utc, Local, DateTime, TimeZone, NaiveDate};
    use chrono_tz::Asia::Tokyo;
    let local: DateTime<Local> = Local::now();
    let tokyo = local.with_timezone(&Tokyo);
    return tokyo.to_string();
}


/*
tests
*/
#[test]
fn get_nowtime_string_test() {
    // TODO : 正規表現を利用したテストを書く
    // assert_eq!(get_nowtime_string(), "2019-09-19 11:12:13.581235812 JST");
}
