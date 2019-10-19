pub mod map;
pub mod postgre;

use super::game::Player;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Progress {
    pub user: Player,
    pub bot: Player,
}

impl Progress {
    pub fn new(user: &Player, bot: &Player) -> Progress {
        return Progress {
            user: user.clone(),
            bot: bot.clone(),
        };
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WinLose {
    pub name: String,
    pub win: i32,
    pub lose: i32,
}

impl WinLose {
    pub fn new(win: i32, lose: i32, name: &str) -> WinLose {
        return WinLose {
            win: win,
            lose: lose,
            name: name.to_string(),
        };
    }
}

pub trait ScoreOperator {
    /// player_name で指定されたプレイヤーの情報を取得する。スコアがまだなかった場合は、None になる
    fn get_progress(&mut self, player_name: &str) -> Option<Progress>;
    /// progress で指定されたプレイヤーの情報を登録する。すでに登録済みの場合は、上書きされる
    fn insert_progress(&mut self, progress: &Progress) -> bool;
    /// player_name で指定されたプレイヤーの情報を削除する。
    fn delete_progress(&mut self, player_name: &str) -> bool;
    /// player_name で指定されたプレイヤーの勝敗を登録する。
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool;
    /// 過去の旗源平の勝敗記録を表示する
    fn get_win_loses(&self) -> Vec<WinLose>;
}
