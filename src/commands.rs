/*
bot が実行するコマンド
*/

use log::{info, warn};
use slack::RtmClient;

/*****************
public functions
******************/

// メンションされた内容をそのまま送り返す
pub fn on_echo(cli: &RtmClient, chid: &String, echo_arg: &String) -> Result<(), failure::Error> {
    info!("called on _echo : args ~ {}", echo_arg);
    if echo_arg.len() > 0 {
        let _ = cli.sender().send_message(chid, echo_arg);
    } else {
        warn!("echo_arg.len() == 0, so can't send echo message to slack");
    }
    return Ok(());
}

// 現在時刻を取得し、chid で指定されたチャンネルに投稿する
pub fn on_nowtime(cli: &RtmClient, chid: &String) -> Result<(), failure::Error> {
    let nowtimestr = get_nowtime_string();
    let _ = cli.sender().send_message(chid, &nowtimestr);
    return Ok(());
}

pub fn on_help(cli: &RtmClient, chid: &String, docs: Vec<&str>) -> Result<(), failure::Error> {
    let docstr = docs
        .iter()
        .fold("".to_string(), |res, doc| format!("{}\n{}", res, doc));
    let _ = cli.sender().send_message(chid, &docstr);
    return Ok(());
}

/*****************
private functions
******************/

fn get_nowtime_string() -> String {
    use chrono::{DateTime, Local};
    use chrono_tz::Asia::Tokyo;
    let local: DateTime<Local> = Local::now();
    let tokyo = local.with_timezone(&Tokyo);
    return tokyo.to_string();
}

/*****************
tests
******************/
#[test]
fn get_nowtime_string_test() {
    use regex::Regex;
    // 意図したフォーマットで現在時刻を表す文字列が取得できているかテスト
    // このフォーマットを想定 : "2019-09-19 11:12:13.581235812 JST"
    let re = Regex::new(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2}) (?P<hour>\d{2}):(?P<min>\d{2}):(?P<sec>\d{2}).(?P<decisec>\d{9}) JST").unwrap();
    let nowtime = get_nowtime_string();
    let _ = re.captures(&nowtime[..]).unwrap();
}
