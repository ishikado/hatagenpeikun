//!
//! 旗源平をbotで実現するモジュール
//!
use super::game::*;
use log::error;
use std::collections::BTreeMap;

const REDIS_HATAGENPEI_PROGRESS_KEY: &str = "hatagenpei_progress";
const HATAGENPEI_INIT_SCORE: i32 = 30;

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
    redis_uri: Option<String>,
    score_map: BTreeMap<String, ScorePair>, // redis が使えないときに利用する
}


trait ScoreOperation {
    fn get_score(player_name : &String) -> ScorePair ;
    fn insert_score(player_name : &String, score_pair : &ScorePair) -> bool;
    fn delete_score(player_name : &String) -> bool ;
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


// TODO 実装する
impl ScoreOperation for ScoresInMap {
    fn get_score(player_name : &String) -> ScorePair {
        return ScorePair::new(0, 0);
    }
    fn insert_score(player_name : &String, score_pair : &ScorePair) -> bool {
        return false;
    }
    fn delete_score(player_name : &String) -> bool {
        return false;
    }
}



impl HatagenpeiController {
    pub fn new(redis_uri: &Option<String>, bot_name: &String) -> HatagenpeiController {
        return HatagenpeiController {
            redis_uri: redis_uri.clone(),
            bot_name: bot_name.clone(),
            score_map: BTreeMap::new(),
        };
    }

    /// 2step旗源平の実行を行う（player -> bot）
    pub fn step(&mut self, player_name: &str) -> Vec<String> {
        let score_pair: &ScorePair;
        // redis から、 player_name を key として、現在のスコアを取り出す
        match &self.redis_uri {
            None => {
                match self.score_map.get(player_name) {
                    None => {
                        self.score_map.insert(
                            player_name.to_string(),
                            ScorePair::new(HATAGENPEI_INIT_SCORE, HATAGENPEI_INIT_SCORE),
                        );
                    }
                    _ => {}
                };
                score_pair = self.score_map.get(player_name).unwrap();
            }
            Some(_uri) => {
                // TODO redisからの取り出しを実装する
                error!("not implementation!");
                panic!("");
            }
        }

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
                        match &self.redis_uri {
                            None => {
                                self.score_map.insert(
                                    player_name.to_string(),
                                    ScorePair::new(p1.score.score, p2.score.score),
                                );
                            }
                            Some(_uri) => {
                                // TODO 実装する
                            }
                        };
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
                    match &self.redis_uri {
                        None => {
                            self.score_map.remove(player_name);
                        }
                        Some(_uri) => {
                            // TODO 実装する
                        }
                    };
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
