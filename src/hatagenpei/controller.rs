//!
//! 旗源平をbotで実現するモジュール
//!
extern crate postgres;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::game::*;
use postgres::{Connection, TlsMode};

// TODO: このあたりの設定は https://docs.rs/config/0.9.3/config/ を使って、Settings.toml から指定できるようにしたい
const DB_HATAGENPEI_PROGRESS_KEY: &str = "hatagenpei_progress";
const DB_HATAGENPEI_WINLOSES_KEY: &str = "hatagenpei_winloses";
const HATAGENPEI_INIT_SCORE: i32 = 29; // 小旗が両替できるように10x(x>=0) + 9 本持ちで開始すること

pub enum DataStore {
    Postgre { uri: String },
    OnMemory,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WinLose {
    pub name: String,
    pub win: i32,
    pub lose: i32,
}

impl WinLose {
    fn new(win: i32, lose: i32, name: &str) -> WinLose {
        return WinLose {
            win: win,
            lose: lose,
            name: name.to_string(),
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Progress {
    user: Player,
    bot: Player,
}

impl Progress {
    fn new(user: &Player, bot: &Player) -> Progress {
        return Progress {
            user: user.clone(),
            bot: bot.clone(),
        };
    }
}

pub struct HatagenpeiController {
    bot_name: String,
    score_operator: Box<dyn ScoreOperation>,
}

trait ScoreOperation {
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

struct ScoresInMap {
    score_map: BTreeMap<String, Progress>,
    winlose_map: BTreeMap<String, WinLose>,
    bot_name: String,
}

struct ScoresInPostgre {
    postgre_uri: String,
    bot_name: String,
}

impl ScoresInMap {
    pub fn new(bot_name: String) -> ScoresInMap {
        return ScoresInMap {
            score_map: BTreeMap::new(),
            winlose_map: BTreeMap::new(),
            bot_name: bot_name,
        };
    }
}

impl ScoresInPostgre {
    pub fn new(postgre_uri: &String, bot_name: String) -> ScoresInPostgre {
        // postgre に接続
        let conn = Connection::connect(&postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // progress管理テーブル作成
        let create_progress_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                    name            VARCHAR NOT NULL,
                    data            VARCHAR NOT NULL
                  )",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        conn.execute(&create_progress_table_query[..], &[])
            .expect("failed to create progress table");

        // winlose 管理テーブル作成
        let create_winlose_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                    name            VARCHAR NOT NULL,
                    data            VARCHAR NOT NULL
                  )",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        conn.execute(&create_winlose_table_query[..], &[])
            .expect("failed to create winlose table");

        return ScoresInPostgre {
            postgre_uri: postgre_uri.clone(),
            bot_name: bot_name,
        };
    }
}

impl ScoreOperation for ScoresInMap {
    fn get_progress(&mut self, player_name: &str) -> Option<Progress> {
        if let Some(progress) = self.score_map.get(player_name) {
            return Some(progress.clone());
        } else {
            return None;
        }
    }

    fn insert_progress(&mut self, progress: &Progress) -> bool {
        self.score_map
            .insert(progress.user.name.to_string(), progress.clone());
        return true;
    }
    fn delete_progress(&mut self, player_name: &str) -> bool {
        self.score_map.remove(player_name);
        return true;
    }
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool {
        let mut win_lose = match self.winlose_map.get(player_name) {
            Some(win_lose) => win_lose.clone(),
            None => WinLose::new(0, 0, player_name),
        };

        if is_player_win {
            win_lose.win += 1;
        } else {
            win_lose.lose += 1;
        }

        self.winlose_map.insert(player_name.to_string(), win_lose);

        return true;
    }
    fn get_win_loses(&self) -> Vec<WinLose> {
        let mut res = vec![];
        for (_, win_lose) in self.winlose_map.iter() {
            res.push(win_lose.clone());
        }
        return res;
    }
}

impl ScoreOperation for ScoresInPostgre {
    fn get_progress(&mut self, player_name: &str) -> Option<Progress> {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");
        // 見つからなかった場合は、insert を実行する
        let select_query = format!(
            "SELECT name, data FROM {} where name = $1",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        let res = conn
            .query(&select_query[..], &[&player_name])
            .expect("failed to select query for get_progress");

        if res.len() == 0 {
            return None;
        }

        // 複数ある場合でも、1つだけ返す
        let r = res.get(0);
        let data: String = r.get(1);
        let progress = serde_json::from_str(&data[..]).expect("failed to serde_json::from_str");
        return Some(progress);
    }

    fn insert_progress(&mut self, progress: &Progress) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // すでに要素が存在している場合は、SQL update
        // そうでない場合は SQL insert を行う
        let select_query = format!(
            "SELECT name, data FROM {} where name = $1",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        let res = conn
            .query(&select_query[..], &[&&progress.user.name[..]])
            .expect("failed to select query for insert_progress");

        let jsonstr = serde_json::to_string(&progress).expect("failed to serde_json::to_string");
        if res.len() == 0 {
            // insert
            let insert_query = format!(
                "INSERT INTO {} (name, data) VALUES ($1, $2)",
                DB_HATAGENPEI_PROGRESS_KEY
            );
            conn.execute(&insert_query[..], &[&&progress.user.name[..], &jsonstr])
                .expect("failed to insert query for insert_progress");
        } else {
            // update
            let update_query = format!(
                "UPDATE {} SET data = $1 WHERE name = $2",
                DB_HATAGENPEI_PROGRESS_KEY
            );
            conn.execute(&update_query[..], &[&jsonstr, &&progress.user.name[..]])
                .expect("failed to update query for insert_progress");
        }
        return true;
    }

    fn delete_progress(&mut self, player_name: &str) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");
        let delete_query = format!("DELETE FROM {} where name = $1", DB_HATAGENPEI_PROGRESS_KEY);
        conn.execute(&delete_query[..], &[&player_name])
            .expect("failed to delete query for delete_progress");

        return true;
    }
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // 勝敗を取得
        let select_query = format!(
            "SELECT name, data, data FROM {} where name = $1",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        let res = conn
            .query(&select_query[..], &[&player_name])
            .expect("failed to select query for update_winloses");

        let mut win_lose = if res.len() == 0 {
            WinLose::new(0, 0, player_name)
        } else {
            let r = res.get(0);
            let data: String = r.get(1);
            serde_json::from_str(&data[..]).expect("failed to serde_json::from_str")
        };

        if is_player_win {
            win_lose.win += 1;
        } else {
            win_lose.lose += 1;
        }

        let s = serde_json::to_string(&win_lose).expect("failed to serde_json::to_string");

        // insert
        if res.len() == 0 {
            let insert_query = format!(
                "INSERT INTO {} (name, data) VALUES ($1, $2)",
                DB_HATAGENPEI_WINLOSES_KEY
            );
            conn.execute(&insert_query[..], &[&player_name, &s])
                .expect("failed to insert query for update_winloses");
        }
        // update
        else {
            let update_query = format!(
                "UPDATE {} SET data = $1 WHERE name = $2",
                DB_HATAGENPEI_WINLOSES_KEY
            );
            conn.execute(&update_query[..], &[&s, &player_name])
                .expect("failed to update query for update_winloses");
        }

        return true;
    }

    fn get_win_loses(&self) -> Vec<WinLose> {
        let mut res = vec![];
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // 勝敗を取得
        let select_query = format!(
            "SELECT name, data, data FROM {}",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        let query_result = conn
            .query(&select_query[..], &[])
            .expect("failed to select query for get_win_loses");

        for row in &query_result {
            let data: String = row.get(1);
            let win_lose = serde_json::from_str(&data[..]).expect("failed to serde_json::from_str");
            res.push(win_lose);
        }
        return res;
    }
}

pub struct StepResult {
    /// HatagenpeiController::step の実行ゲームログ
    pub logs: Vec<String>,
    /// ゲームが終了したかどうか
    pub is_over: bool,
    /// この step 呼び出しで、ゲームが開始したかどうか
    pub is_start: bool
}

impl HatagenpeiController {
    pub fn new(data_store: &DataStore, bot_name: &String) -> HatagenpeiController {
        let score_operator: Box<dyn ScoreOperation> = match data_store {
            DataStore::Postgre { uri } => Box::new(ScoresInPostgre::new(uri, bot_name.clone())),
            DataStore::OnMemory => Box::new(ScoresInMap::new(bot_name.clone())),
        };

        return HatagenpeiController {
            bot_name: bot_name.clone(),
            score_operator: score_operator,
        };
    }

    /// 過去の勝敗を取得
    pub fn get_win_loses(&self) -> Vec<WinLose> {
        return self.score_operator.get_win_loses().clone();
    }

    /// 2step旗源平の実行を行う（player -> bot）
    pub fn step(&mut self, player_name: &str) -> StepResult {
        let seed = rand::random::<u64>();
        let mut is_start = false;
        // 現在の状態でゲームを行う
        let progress = match self.score_operator.get_progress(player_name) {
            Some(progress) => progress,
            None => {
                // 初期 progress を作成
                let progress = Progress::new(
                    &Player::new(
                        player_name.to_string(),
                        Score {
                            score: HATAGENPEI_INIT_SCORE,
                            matoi: true,
                        },
                        Score {
                            score: 0,
                            matoi: false,
                        },
                    ),
                    &Player::new(
                        self.bot_name.clone(),
                        Score {
                            score: HATAGENPEI_INIT_SCORE,
                            matoi: true,
                        },
                        Score {
                            score: 0,
                            matoi: false,
                        },
                    ),
                );
                is_start = true;
                // 登録
                self.score_operator.insert_progress(&progress);
                progress
            }
        };

        let mut game = Hatagenpei::new(progress.user, progress.bot, PlayerTurn::Player1, seed);
        // game.next() の戻り値から、ゲームログ文字列を構築する
        let mut logstr = vec![];
        let mut is_over = false;

        // (i == 0) => user play, (i == 1) => bot play
        for i in 0..2 {
            // unwrap できない場合、予期しない状態になっている可能性があるので panic する
            let game_log = game.next().unwrap();

            let turn_player_name = match game_log.player_turn {
                PlayerTurn::Player1 => game_log.player1.name.clone(),
                PlayerTurn::Player2 => game_log.player2.name.clone(),
            };

            logstr.push(format!("# {} の番", turn_player_name).to_string());
            logstr.push("## サイコロの結果".to_string());

            for cmd in &game_log.commands {
                logstr.push(format!("- {}", cmd.explain.to_string()));
            }

            logstr.push("".to_string());
            logstr.push("## 旗状況".to_string());

            for player in [&(game_log.player1), &(game_log.player2)].iter() {
                logstr.push(format!("- {}", player.name));
                logstr.push(format!(
                    "   - 自分の旗 【{}】",
                    player.my_score.to_string()
                ));
                logstr.push(format!(
                    "   - 取った旗 【{}】",
                    player.got_score.to_string()
                ));
            }

            logstr.push("".to_string());

            match game_log.game_state {
                GameState::YetPlaying => {
                    // ループ終了時
                    if i == 1 {
                        // スコアの再登録
                        self.score_operator
                            .insert_progress(&Progress::new(&game_log.player1, &game_log.player2));
                    }
                }
                win_player => {
                    let win_player_name = match win_player {
                        GameState::Player1Win => player_name.to_string(),
                        GameState::Player2Win => self.bot_name.clone(),
                        GameState::YetPlaying => panic!("unexpected!"),
                    };

                    logstr.push(format!("{} の勝ち", win_player_name));
                    logstr.push("".to_string());

                    // ゲームが終わったので、進行状態を削除する
                    self.score_operator.delete_progress(player_name);

                    // 勝敗を書く
                    self.score_operator
                        .update_winloses(player_name, win_player == GameState::Player1Win);

                    is_over = true;
                    break;
                }
            }
        }
        return StepResult {
            logs: logstr,
            is_over: is_over,
            is_start: is_start
        };
    }
}
