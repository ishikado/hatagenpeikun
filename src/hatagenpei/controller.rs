//!
//! 旗源平をbotで実現するモジュール
//!
extern crate redis;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use redis::*;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::game::*;

// TODO: このあたりの設定は https://docs.rs/config/0.9.3/config/ を使って、Settings.toml から指定できるようにしたい
const DB_HATAGENPEI_PROGRESS_KEY: &str = "hatagenpei_progress";
const DB_REDIS_HATAGENPEI_WINLOSES_KEY: &str = "hatagenpei_winloses";
const HATAGENPEI_INIT_SCORE: i32 = 29; // 小旗が両替できるように10x(x>=0) + 9 本持ちで開始すること

pub enum DataStore {
    Redis { uri: String },
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
    /// player_name で指定されたプレイヤーの情報を取得する。スコアがまだなかった場合は、初期値が insert されたあと、取得される。
    fn get_progress(&mut self, player_name: &str) -> Progress;
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

struct ScoresInRedis {
    redis_uri: String,
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

impl ScoresInRedis {
    pub fn new(redis_uri: &String, bot_name: String) -> ScoresInRedis {
        return ScoresInRedis {
            redis_uri: redis_uri.clone(),
            bot_name: bot_name,
        };
    }
}

impl ScoresInPostgre {
    pub fn new(postgre_uri: &String, bot_name: String) -> ScoresInPostgre {
        return ScoresInPostgre {
            postgre_uri: postgre_uri.clone(),
            bot_name: bot_name,
        };
    }
}

impl ScoreOperation for ScoresInMap {
    fn get_progress(&mut self, player_name: &str) -> Progress {
        match self.score_map.get(player_name) {
            None => {
                self.insert_progress(&Progress::new(
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
                ));
            }
            _ => {}
        };
        // 空の場合は上で中身を insert しているので、必ず取り出せる
        return self.score_map.get(player_name).unwrap().clone();
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

impl ScoreOperation for ScoresInRedis {
    //!
    //! redis接続周りで unwrap を多様して、エラーになった場合 panic させる作りになっている
    //! これは、エラーハンドリングをちゃんと行ったとしても、特にリカバリができるわけでもないため
    //! どちらでも良いなら、panic させてしまう方が実装的には楽
    //!
    fn get_progress(&mut self, player_name: &str) -> Progress {
        // TODO: エラーハンドリング

        //  TODO: DBへの接続は、new するときにやってしまったほうがよいかも
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();
        let get_result: RedisResult<String> = con.hget(DB_HATAGENPEI_PROGRESS_KEY, player_name);
        let progress;

        // スコアを json 形式で取り出す
        match get_result {
            Ok(json) => {
                progress = serde_json::from_str(&json[..]).unwrap();
            }
            // 取り出せなかった場合、insert しておく
            Err(_) => {
                progress = Progress::new(
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

                if self.insert_progress(&progress) {
                } else {
                    // TODO insert_progress に失敗した場合はエラー扱いにしたい
                }
            }
        }
        return progress;
    }

    fn insert_progress(&mut self, progress: &Progress) -> bool {
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();
        let s = serde_json::to_string(progress).unwrap();
        let _: () = con
            .hset(DB_HATAGENPEI_PROGRESS_KEY, progress.user.name.clone(), s)
            .unwrap();
        return true;
    }

    fn delete_progress(&mut self, player_name: &str) -> bool {
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();
        let _res: i32 = con
            .hdel(DB_HATAGENPEI_PROGRESS_KEY, player_name)
            .unwrap();
        return true;
    }
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool {
        // まずスコアテーブルを取り出し、値を確認する
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();

        let get_result: RedisResult<String> = con.hget(DB_REDIS_HATAGENPEI_WINLOSES_KEY, player_name);

        let mut win_lose = match get_result {
            Ok(json) => serde_json::from_str(&json[..]).unwrap(),
            // 取り出せなかった場合、insert しておく
            Err(_) => WinLose::new(0, 0, player_name),
        };

        if is_player_win {
            win_lose.win += 1;
        } else {
            win_lose.lose += 1;
        }

        // 勝敗テーブルに勝敗を書き込み
        let s = serde_json::to_string(&win_lose).unwrap();
        let _: () = con
            .hset(DB_REDIS_HATAGENPEI_WINLOSES_KEY, player_name, s)
            .unwrap();

        return true;
    }

    fn get_win_loses(&self) -> Vec<WinLose> {
        let mut res = vec![];
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();

        let btreemap_result: RedisResult<BTreeMap<String, String>> =
            con.hgetall(DB_REDIS_HATAGENPEI_WINLOSES_KEY);

        match btreemap_result {
            Ok(win_lose_map) => {
                for (_, jsonstr) in win_lose_map.iter() {
                    let win_lose: WinLose = serde_json::from_str(jsonstr).unwrap();
                    res.push(win_lose);
                }
            }
            Err(_) => {
                // key が取り出せない場合は、何もしない
            }
        }
        return res;
    }
}


// TODO 実装する
impl ScoreOperation for ScoresInPostgre {
    fn get_progress(&mut self, player_name: &str) -> Progress {
        Progress::new(
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
                player_name.to_string(),
                Score {
                    score: HATAGENPEI_INIT_SCORE,
                    matoi: true,
                },
                Score {
                    score: 0,
                    matoi: false,
                },
            ))
    }

    fn insert_progress(&mut self, progress: &Progress) -> bool {
        return true;
    }

    fn delete_progress(&mut self, player_name: &str) -> bool {
        return true;
    }
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool {
        return true;
    }

    fn get_win_loses(&self) -> Vec<WinLose> {
        let mut res = vec![];
        return res;
    }
}


pub struct StepResult {
    /// HatagenpeiController::step の実行ゲームログ
    pub logs: Vec<String>,
    /// ゲームが終了したかどうか
    pub is_over: bool,
}

impl HatagenpeiController {
    pub fn new(data_store: &DataStore, bot_name: &String) -> HatagenpeiController {
        let score_operator: Box<dyn ScoreOperation> = match data_store {
            DataStore::Redis { uri } => Box::new(ScoresInRedis::new(uri, bot_name.clone())),
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
        // 現在の状態でゲームを行う
        let progress = self.score_operator.get_progress(player_name);
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
        };
    }
}
