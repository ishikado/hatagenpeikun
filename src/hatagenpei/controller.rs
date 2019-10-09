//!
//! 旗源平をbotで実現するモジュール
//!
extern crate redis;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use log::error;
use redis::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use std::collections::BTreeMap;

use super::game::*;

// TODO このあたりの設定は conf か 引数 で指定できるようにしたい
const REDIS_HATAGENPEI_PROGRESS_KEY: &str = "hatagenpei_progress";
const HATAGENPEI_INIT_SCORE: i32 = 30;

#[derive(Clone, Serialize, Deserialize)]
struct ScorePair {
    player_score: i32,
    bot_score: i32,
}

impl ScorePair {
    fn new(player_score: i32, bot_score: i32) -> ScorePair {
        return ScorePair {
            player_score: player_score,
            bot_score: bot_score,
        };
    }
}

pub struct HatagenpeiController {
    bot_name: String,
    score_operator : Box<dyn ScoreOperation> 
}


trait ScoreOperation {
    fn get_score(&mut self, player_name : &str) -> ScorePair ;
    fn insert_score(&mut self, player_name : &str, score_pair : &ScorePair) -> bool;
    fn delete_score(&mut self, player_name : &str) -> bool ;
}

struct ScoresInMap {
    score_map : BTreeMap<String, ScorePair>
}

struct ScoresInRedis {
    redis_uri: String,
}

impl ScoresInMap {
    pub fn new() -> ScoresInMap {
        return ScoresInMap{score_map : BTreeMap::new() };
    }
}

impl ScoresInRedis {
    pub fn new(redis_uri : &String) -> ScoresInRedis {
        return ScoresInRedis{redis_uri : redis_uri.clone()};
    }
}


impl ScoreOperation for ScoresInMap {
    fn get_score(&mut self, player_name : &str) -> ScorePair {
        match self.score_map.get(player_name) {
            None => {
                self.insert_score(player_name, &ScorePair::new(HATAGENPEI_INIT_SCORE, HATAGENPEI_INIT_SCORE));
            }
            _ => {}
        };
        // 空の場合は上で中身を insert しているので、必ず取り出せる
        return self.score_map.get(player_name).unwrap().clone();
    }

    fn insert_score(&mut self, player_name : &str, score_pair : &ScorePair) -> bool {
        self.score_map.insert(
            player_name.to_string(),
            score_pair.clone()
        );
        return true;
    }
    fn delete_score(&mut self, player_name : &str) -> bool {
        self.score_map.remove(player_name);
        return true;
    }
}

impl ScoreOperation for ScoresInRedis {
    fn get_score(&mut self, player_name : &str) -> ScorePair {
        // TODO: エラーハンドリング
        let client = Client::open(&self.redis_uri[..]).unwrap();
        let mut con = client.get_connection().unwrap();

        // スコアを json 形式で取り出す
        let json: String = con.hget(REDIS_HATAGENPEI_PROGRESS_KEY, player_name).unwrap();
        let score: ScorePair = serde_json::from_str(&json[..]).unwrap();

        // TODO: スコアが存在しない場合の対応

        return score;
    }

    fn insert_score(&mut self, player_name : &str, score_pair : &ScorePair) -> bool {
        return false;
    }
    fn delete_score(&mut self, player_name : &str) -> bool {
        return false;
    }
}




impl HatagenpeiController {
    pub fn new(redis_uri: &Option<String>, bot_name: &String) -> HatagenpeiController {

        let score_operator : Box<dyn ScoreOperation> = match redis_uri {
            None => {
                Box::new(ScoresInMap::new())
            }
            Some(uri) => {
                Box::new(ScoresInRedis::new(uri))
            }
        };

        return HatagenpeiController {
            bot_name: bot_name.clone(),
            score_operator : score_operator
        };
    }

    /// 2step旗源平の実行を行う（player -> bot）
    pub fn step(&mut self, player_name: &str) -> Vec<String> {        

        let score_pair = self.score_operator.get_score(player_name);

        // 現在の状態でゲームを行う
        let mut game = Hatagenpei::new(
            Player::new(
                player_name,
                Score {
                    score: score_pair.player_score,
                },
            ),
            Player::new(
                &self.bot_name[..],
                Score {
                    score: score_pair.bot_score,
                },
            ),
            PlayerTurn::Player1,
        );

        let mut finres = vec![];
        for i in 0..2 {
            let mut res = game.next();
            finres.append(&mut res);

            match game.get_victory_or_defeat() {
                Ok(VictoryOrDefat::YetPlaying) => {
                    // ループ終了時
                    if i == 1 {
                        let (p1, p2) = game.get_score();
                        // スコアの再登録
                        self.score_operator.insert_score(player_name, &ScorePair::new(p1.score.score, p2.score.score));
                    }
                }
                Ok(win_player) => {
                    let win_player_name = match win_player {
                        VictoryOrDefat::Player1Win => player_name.to_string(),
                        VictoryOrDefat::Player2Win => self.bot_name.clone(),
                        VictoryOrDefat::YetPlaying => panic!("unexpected!"),
                    };

                    finres.push(format!("{} is win!!", win_player_name));

                    // ゲームが終わったので、進行状態を削除する
                    self.score_operator.delete_score(player_name);

                    break;
                }
                Err(err) => {
                    // ここを通ったら異常なので panic する
                    // redis の場合は key を消してしまったほうがいいかもしれない
                    error!("error occured!, error = {:?}", err);
                    panic!("");
                }
            }
        }
        return finres;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn controller_tests() {
        // controller を動作させ、ちゃんと状態が保存されているか見る
        use crate::hatagenpei::controller::*;

        let mut ins = HatagenpeiController::new(&None, &"hatagenpeikun".to_string());

        for _ in 0..2 {
            let res = ins.step(&"rust".to_string());
            for l in res {
                println!("{:?}", l);
            }
        }
    }
}
