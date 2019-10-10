//!
//! 旗源平を実現するモジュール
//!

extern crate rand;
use rand::prelude::*;
use std::mem;
use std::string::ToString;
use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct Score {
    pub score: i32,
    pub matoi: bool,
}


impl Score {
    fn to_string(&self) -> String {
        let obata = self.score / 50;
        let chubata = (self.score % 50) / 10;
        let kobata = (self.score % 50) % 10;
        return format!(
            "大旗 : {} 本、中旗 : {} 本、小旗 : {} 本",
            obata, chubata, kobata
        )
        .to_string();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub my_score: Score,
    pub got_score: Score,
    pub name: String,
}

impl Player {
    pub fn new(name: String, my_score: Score, got_score : Score) -> Player {
        return Player {
            my_score: my_score,
            got_score: got_score,
            name: name,
        };
    }
}

#[derive(PartialEq)]
pub enum PlayerTurn {
    Player1,
    Player2,
}

#[derive(PartialEq)]
pub enum VictoryOrDefeat {
    Player1Win,
    Player2Win,
    YetPlaying,
}

pub struct Hatagenpei {
    pub player1: Player,
    pub player2: Player,
    pub turn: PlayerTurn,
}

#[derive(Debug)]
pub enum HatagenPeiError {
    Unexpected,
}

#[derive(Clone)]
struct HatagenpeiCommand {
    dice1: u8,
    dice2: u8,
    point: i32,            // + なら もらう、- ならあげる
    again: bool,           // true ならもういちどサイコロを
    explain: &'static str, // 説明文
}

const HATAGENPEICOMMANDS: [HatagenpeiCommand; 21] = [
    HatagenpeiCommand {
        dice1: 1,
        dice2: 1,
        point: 2,
        again: true,
        explain: "１  １	ちんちんかもかも　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 2,
        dice2: 2,
        point: 2,
        again: true,
        explain: "２  ２	にゃあにゃあ	　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 3,
        dice2: 3,
        point: 2,
        again: true,
        explain: "３  ３	さざなみ	　　　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 4,
        dice2: 4,
        point: 2,
        again: true,
        explain: "４  ４	しゅうじゅう	　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 5,
        dice2: 5,
        point: 2,
        again: true,
        explain: "５  ５	ごんご	　　　　　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 6,
        dice2: 6,
        point: 2,
        again: true,
        explain: "６  ６	じょうろく	　　　　小旗２本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 1,
        dice2: 2,
        point: 1,
        again: false,
        explain: "１  ２	ちんに	　　　　　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 1,
        dice2: 3,
        point: 1,
        again: false,
        explain: "１  ３	ちんさん　　	　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 1,
        dice2: 4,
        point: 1,
        again: false,
        explain: "１  ４	ちんし	　　　　　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 1,
        dice2: 5,
        point: 10,
        again: true,
        explain: "１  ５	うめがいち　　	　　中旗１本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 1,
        dice2: 6,
        point: 10,
        again: true,
        explain: "１  ６	ちんろく	　　　　中旗１本もらう　さいころを続けて振れる",
    },
    HatagenpeiCommand {
        dice1: 2,
        dice2: 3,
        point: 0,
        again: false,
        explain: "２  ３	にさまのかんかんど	旗の移動なし",
    },
    HatagenpeiCommand {
        dice1: 2,
        dice2: 4,
        point: -10,
        again: false,
        explain: "２  ４	しのに	　　　　　　中旗１本返す",
    },
    HatagenpeiCommand {
        dice1: 2,
        dice2: 5,
        point: 0,
        again: false,
        explain: "２  ５	ごにごに	　　　　旗の移動なし",
    },
    HatagenpeiCommand {
        dice1: 2,
        dice2: 6,
        point: 1,
        again: false,
        explain: "２  ６	ろくに	　　　　　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 3,
        dice2: 4,
        point: 0,
        again: false,
        explain: "３  ４	しさまのかんかんど　旗の移動なし",
    },
    HatagenpeiCommand {
        dice1: 3,
        dice2: 5,
        point: 0,
        again: false,
        explain: "３  ５	ごさまのかんかんど　旗の移動なし",
    },
    HatagenpeiCommand {
        dice1: 3,
        dice2: 6,
        point: 1,
        again: false,
        explain: "３  ６	ろくさん	　　　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 4,
        dice2: 5,
        point: 0,
        again: false,
        explain: "４  ５	ごっしりはなかみ　　旗の移動なし",
    },
    HatagenpeiCommand {
        dice1: 4,
        dice2: 6,
        point: 1,
        again: false,
        explain: "４  ６	しろく	　　　　　　小旗１本もらう",
    },
    HatagenpeiCommand {
        dice1: 5,
        dice2: 6,
        point: 1,
        again: false,
        explain: "５  ６	ごろく	　　　　　　小旗１本もらう",
    },
];

impl Hatagenpei {
    /// Hatagenpei インスタンスを作成する
    pub fn new(
        player1: Player,
        player2: Player,
        first_player: PlayerTurn,
    ) -> Hatagenpei {
        return Hatagenpei {
            player1: player1,
            player2: player2,
            turn: first_player,
        };
    }

    /// 1ターン進める。
    /// サイコロの振り直しが発生した場合、振り直しを行う。
    /// 戻り値で、実行ログを返す
    pub fn next(&mut self) -> Vec<String> {
        let mut res = vec![];

        {
            let next_turn;
            let turn_player_name;

            match self.turn {
                PlayerTurn::Player1 => {
                    turn_player_name = self.player1.name.clone();
                    next_turn = PlayerTurn::Player2;
                }
                PlayerTurn::Player2 => {
                    turn_player_name = self.player2.name.clone();
                    next_turn = PlayerTurn::Player1;
                }
            }

            // まだプレイ中でなければならない
            match self.get_victory_or_defeat() {
                Ok(VictoryOrDefeat::YetPlaying) => {}
                _ => {
                    return res;
                }
            }

            res.push(format!("# {} の番", turn_player_name).to_string());

            res.push("## サイコロの結果".to_string());
            loop {
                match self.get_victory_or_defeat() {
                    Ok(VictoryOrDefeat::YetPlaying) => {
                        // まだプレイ中の場合のみダイスを振る
                        let cmd = Self::diceroll();

                        // 旗を返すプレイヤーを決定
                        let (send_player, got_player) = 
                            if (cmd.point > 0) as i32 ^ (self.turn == PlayerTurn::Player1) as i32 > 0 {
                                (&mut self.player1, &mut self.player2)
                            } else {
                                (&mut self.player2, &mut self.player1)
                            };

                        let v = std::cmp::min(cmd.point.abs(), send_player.my_score.score);
                        send_player.my_score.score -= v;
                        got_player.got_score.score += v;

                        res.push(format!("- {}", cmd.explain.to_string()));

                        // もう一度振れないなら終了
                        if !cmd.again {
                            break;
                        }
                    }
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
            self.turn = next_turn;
        }

        res.push("".to_string());
        res.push("## 旗状況".to_string());

        for player in [&self.player1, &self.player2].iter() {
            res.push(format!("- {}", player.name));
            res.push(format!("   - 自分の旗 【{}】", player.my_score.to_string()));
            res.push(format!("   - 取った旗 【{}】", player.got_score.to_string()));
        }

        res.push("".to_string());

        return res;
    }

    /// (player1, player2) というタプルで、現在の Player ごとのスコアを取得する。
    pub fn get_score(&self) -> (&Player, &Player) {
        return (&self.player1, &self.player2);
    }

    pub fn get_victory_or_defeat(self: &Self) -> Result<VictoryOrDefeat, HatagenPeiError> {
        return Self::get_victory_or_defeat_(&self.player1, &self.player2);
    }

    /// サイコロを振り、行うコマンドを返す
    fn diceroll() -> HatagenpeiCommand {
        // 乱数でサイコロの目を決める
        let mut d1 = (rand::random::<u8>() % 6) + 1;
        let mut d2 = (rand::random::<u8>() % 6) + 1;
        if d1 > d2 {
            mem::swap(&mut d1, &mut d2);
        }

        let cmd = HATAGENPEICOMMANDS
            .iter()
            .find(|cmd| {
                return cmd.dice1 == d1 && cmd.dice2 == d2;
            })
            .unwrap();

        return cmd.clone();
    }

    fn get_victory_or_defeat_(
        player1: &Player,
        player2: &Player,
    ) -> Result<VictoryOrDefeat, HatagenPeiError> {
        if player1.my_score.score == 0 && player2.my_score.score > 0 {
            return Ok(VictoryOrDefeat::Player2Win);
        } else if player1.my_score.score > 0 && player2.my_score.score == 0 {
            return Ok(VictoryOrDefeat::Player1Win);
        } else if player1.my_score.score > 0 && player2.my_score.score > 0 {
            return Ok(VictoryOrDefeat::YetPlaying);
        } else {
            return Err(HatagenPeiError::Unexpected);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hatagenpei_tests() {
        // TODO : print せずに、機械的に比較できるテストを実装したい

        use crate::hatagenpei::game::*;

        let first_player_name = "first";
        let second_player_name = "second";
        let initial_score = 30;

        let p1 = Player::new(first_player_name.to_string(), Score{score : initial_score, matoi : true}, Score{score : 0, matoi : false});
        let p2 = Player::new(second_player_name.to_string(), Score{score : initial_score, matoi : true}, Score{score : 0, matoi : false});

        let mut hg = Hatagenpei::new(p1, p2, PlayerTurn::Player1);

        let mut call_next_count = 0;

        loop {
            let v = hg.next();
            call_next_count += 1;

            for i in v {
                println!("{:?}", i);
            }

            println!("");

            match hg.get_victory_or_defeat() {
                Ok(VictoryOrDefeat::YetPlaying) => {
                }
                Ok(VictoryOrDefeat::Player1Win) => {
                    println!("{} win!!", first_player_name);
                    break;
                }
                Ok(VictoryOrDefeat::Player2Win) => {
                    println!("{} win!!", second_player_name);
                    break;
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
        println!("call_next_count = {}", call_next_count);
    }
}
