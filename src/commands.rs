/*
bot が実行するコマンド
*/

use crate::hatagenpei::controller::*;
use log::{info, warn};
use slack::RtmClient;

/*****************
public functions
******************/

pub fn on_hatagenpei(
    cli: &RtmClient,
    controller: &mut Option<HatagenpeiController>,
    message_user_name: &String,
    message_user_id: &String,
    chid: &String,
) -> Result<(), failure::Error> {
    info!("called on_hatagenpei, user_id = {}", message_user_id);
    match controller {
        Some(controller) => {
            let res = controller.step(message_user_name);

            let prefix = if res.is_start {
                "旗源平を始めるげん!\n\n"
            } else {
                ""
            };

            let joined_logs = [prefix, "```", &res.logs.join("\n"), "```"].concat();

            let _ = cli.sender().send_message(chid, &joined_logs);
        }
        None => {
            // do nothing
        }
    }

    return Ok(());
}

pub fn on_hatagenpei_winloses(
    cli: &RtmClient,
    controller: &mut Option<HatagenpeiController>,
    _message_user_name: &String,
    chid: &String,
) -> Result<(), failure::Error> {
    info!("called on_hatagenpei_winloses");
    match controller {
        Some(controller) => {
            let mut s = "```".to_string();
            s.push_str("# 勝敗\n");
            for win_lose in controller.get_win_loses() {
                let escaped_name = escape_name(&win_lose.name);
                s.push_str(
                    &format!(
                        "- {} 【{}勝 {}敗】\n",
                        escaped_name, win_lose.win, win_lose.lose
                    )
                    .to_string(),
                );
            }
            s.push_str("```");
            let _ = cli.sender().send_message(chid, &s);
        }
        None => {
            // do nothing
        }
    }

    return Ok(());
}

// ﾌﾟﾙﾙﾙ に反応する
pub fn on_purururu(cli: &RtmClient, chid: &String, text: &String) -> Result<(), failure::Error> {
    info!("called on_purururu, text = {}", text);
    if let Some(_) = text.find("ﾌﾟﾙﾙﾙ") {
        let _ = cli.sender().send_message(chid, "ﾌﾟﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙﾙ！");
    }
    Ok(())
}

// メンションされた内容をそのまま送り返す
pub fn on_echo(cli: &RtmClient, chid: &String, echo_arg: &String) -> Result<(), failure::Error> {
    info!("called on_echo, args = {}", echo_arg);
    if echo_arg.len() > 0 {
        let _ = cli.sender().send_message(chid, echo_arg);
    } else {
        warn!("echo_arg.len() == 0, so can't send echo message to slack");
    }
    return Ok(());
}

// 現在時刻を取得し、chid で指定されたチャンネルに投稿する
pub fn on_nowtime(cli: &RtmClient, chid: &String) -> Result<(), failure::Error> {
    info!("called on_nowtime");
    let nowtimestr = get_nowtime_string();
    let _ = cli.sender().send_message(chid, &nowtimestr);
    return Ok(());
}

pub fn on_help(cli: &RtmClient, chid: &String, docs: Vec<&str>) -> Result<(), failure::Error> {
    info!("called on_help");
    let anotated = ["```", &docs.join("\n"), "```"].concat();
    let _ = cli.sender().send_message(chid, &anotated);
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

fn escape_name(name: &str) -> String {
    let (_, escaped_name) =
        name.chars()
            .into_iter()
            .fold((0, "".to_string()), |(counter, mut tmp_name), ch| {
                tmp_name.push(ch);
                if counter == 0 {
                    tmp_name.push('.');
                    (counter + 1, tmp_name)
                } else {
                    (counter + 1, tmp_name)
                }
            });
    return escaped_name;
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
