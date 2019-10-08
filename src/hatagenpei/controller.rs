//!
//! 旗源平をbotで実現するモジュール
//!
use std::collections::BTreeMap;
use super::game::*;
use log::{error};


const REDIS_HATAGENPEI_PROGRESS_KEY : &str = "hatagenpei_progress";
const HATAGENPEI_INIT_SCORE : i32 = 30;


struct ScorePair {
    player_score : i32,
    bot_score : i32
}

impl ScorePair {
    fn new(player_score : i32, bot_score : i32) -> ScorePair {
        return ScorePair{player_score : player_score, 
                         bot_score : bot_score};
    }
}

pub struct HatagenpeiController{
    bot_name : String,
    redis_uri : Option<String>,
    score_map : BTreeMap<String, ScorePair> // redis が使えないときに利用する
}

impl HatagenpeiController {
    
    pub fn new(redis_uri : &Option<String>, bot_name : &String) -> HatagenpeiController {
        return HatagenpeiController{redis_uri : redis_uri.clone(), 
                                    bot_name : bot_name.clone(), 
                                    score_map : BTreeMap::new()};
    }

    /// 1step旗源平の実行を行う
    pub fn step(&mut self, player_name : &String) -> Vec<String> {
        let score_pair : &ScorePair;
        // redis から、 player_name を key として、現在のスコアを取り出す
        match &self.redis_uri {
            None => {
                match self.score_map.get(player_name) {
                    None => {
                        self.score_map.insert(player_name.clone(), 
                                              ScorePair::new(HATAGENPEI_INIT_SCORE,
                                                             HATAGENPEI_INIT_SCORE));
                    }
                    _ => {}
                };
                score_pair = self.score_map.get(player_name).unwrap();
            },
            Some(_uri) => {
                // TODO 実装する
                error!("not implementation!");
                panic!("");
            }
        }
        
        // 現在の状態でゲームを行う
        let mut game = Hatagenpei::new(Player::new(player_name.clone(), Score{score : score_pair.player_score}),
                                       Player::new(self.bot_name.clone(), Score{score : score_pair.bot_score}),
                                       PlayerTurn::Player1);
        
        let res = game.next();
        
        match game.get_victory_or_defeat()  {
            Ok(VictoryOrDefat::YetPlaying) => {
            },
            Ok(_) => {
                // ゲームが終わったので、進行状態を削除する
                match &self.redis_uri {
                    None => {
                        self.score_map.remove(player_name);
                    },
                    Some(_uri) => {
                        // TODO 実装する
                    }
                }
            },
            Err(err) => {
                // ここを通ったら異常なので panic する
                error!("error occured!, error = {:?}", err);
                panic!("");
            }
        }
        
        return res;
    }

}



#[cfg(test)]
mod tests {
    #[test]
    fn controller_tests() {
        // controller を動作させ、ちゃんと状態が保存されているか見る
        // TODO 実装する
    }
}
