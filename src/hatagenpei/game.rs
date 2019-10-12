//!
//! 旗源平を実現するモジュール
//!

extern crate rand;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::mem;
use std::string::ToString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Score {
    /// 現在所持している旗の本数
    pub score: i32,
    /// 現在まといを所持しているか（まといがなくなると負ける）
    pub matoi: bool,
}

// ゲームのログ情報
#[derive(Debug)]
pub struct GameLog {
    /// 今回ゲームを実行したプレイヤー
    pub player_turn: PlayerTurn,
    /// 実行したコマンド
    pub commands: Vec<HatagenpeiCommand>,
    /// commands をすべて実行した後の、player1の情報
    pub player1: Player,
    /// commands をすべて実行した後の、player2の情報
    pub player2: Player,
    /// 現在のゲーム状況
    pub game_state: GameState,
}

impl Score {
    pub fn to_string(&self) -> String {
        let obata = self.score / 50;
        let chubata = (self.score % 50) / 10;
        let kobata = (self.score % 50) % 10;
        let m = self.matoi as i32;
        return format!(
            "まとい : {} 本、 大旗 : {} 本、中旗 : {} 本、小旗 : {} 本",
            m, obata, chubata, kobata
        )
        .to_string();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub my_score: Score,
    pub got_score: Score,
    pub name: String,
}

impl Player {
    pub fn new(name: String, my_score: Score, got_score: Score) -> Player {
        return Player {
            my_score: my_score,
            got_score: got_score,
            name: name,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerTurn {
    Player1,
    Player2,
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    Player1Win,
    Player2Win,
    YetPlaying,
}

pub struct Hatagenpei {
    player1: Player,
    player2: Player,
    turn: PlayerTurn,
    rng: rand_xoshiro::Xoshiro256StarStar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HatagenpeiCommand {
    pub dice1: u8,
    pub dice2: u8,
    pub point: i32,            // + なら もらう、- ならあげる
    pub again: bool,           // true ならもういちどサイコロを
    pub explain: &'static str, // 説明文
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
        seed: u64,
    ) -> Hatagenpei {
        return Hatagenpei {
            player1: player1,
            player2: player2,
            turn: first_player,
            rng: rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed),
        };
    }

    /// 1ターン進める。
    /// サイコロの振り直しが発生した場合、振り直しを行う。
    /// 戻り値で、実行ログを返す
    pub fn next(&mut self) -> Option<GameLog> {
        let mut commands = Vec::new();
        let next_turn;

        {
            match self.turn {
                PlayerTurn::Player1 => {
                    next_turn = PlayerTurn::Player2;
                }
                PlayerTurn::Player2 => {
                    next_turn = PlayerTurn::Player1;
                }
            }

            // まだプレイ中でなければならない
            match self.get_game_state() {
                GameState::YetPlaying => {}
                _ => {
                    return None;
                }
            }

            loop {
                match self.get_game_state() {
                    GameState::YetPlaying => {
                        // まだプレイ中の場合のみダイスを振る
                        let cmd = Self::diceroll(&mut self.rng);

                        commands.push(cmd.clone());

                        // 旗を返すプレイヤーを決定
                        let (send_player, got_player) = if (cmd.point > 0) as i32
                            ^ (self.turn == PlayerTurn::Player1) as i32
                            > 0
                        {
                            (&mut self.player1, &mut self.player2)
                        } else {
                            (&mut self.player2, &mut self.player1)
                        };

                        {
                            // TOOD: このあたりのやり取りをもうすこしきれいにしたい

                            // まといのやり取り
                            if cmd.point.abs() > send_player.my_score.score {
                                send_player.my_score.matoi = false;
                                got_player.got_score.matoi = true;
                            }

                            // 旗のやり取り
                            let v = std::cmp::min(cmd.point.abs(), send_player.my_score.score);
                            send_player.my_score.score -= v;
                            got_player.got_score.score += v;
                        }

                        // もう一度振れないなら終了
                        if !cmd.again {
                            break;
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        let game_log = GameLog {
            player1: self.player1.clone(),
            player2: self.player2.clone(),
            commands: commands,
            player_turn: self.turn.clone(),
            game_state: self.get_game_state(),
        };

        self.turn = next_turn;

        return Some(game_log);
    }

    fn get_game_state(self: &Self) -> GameState {
        if self.player1.got_score.matoi {
            return GameState::Player1Win;
        } else if self.player2.got_score.matoi {
            return GameState::Player2Win;
        } else {
            return GameState::YetPlaying;
        }
    }

    /// サイコロを振り、行うコマンドを返す
    fn diceroll(rng: &mut rand_xoshiro::Xoshiro256StarStar) -> HatagenpeiCommand {
        // 乱数でサイコロの目を決める
        let mut d1 = (rng.gen::<u8>() % 6) + 1;
        let mut d2 = (rng.gen::<u8>() % 6) + 1;
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn hatagenpei_tests() {
        use crate::hatagenpei::game::*;

        let mut game = Hatagenpei::new(
            Player::new(
                "alice".to_string(),
                Score {
                    score: 10,
                    matoi: true,
                },
                Score {
                    score: 0,
                    matoi: false,
                },
            ),
            Player::new(
                "bob".to_string(),
                Score {
                    score: 10,
                    matoi: true,
                },
                Score {
                    score: 0,
                    matoi: false,
                },
            ),
            PlayerTurn::Player1,
            123,
        );

        {
            let game_log = game.next().unwrap();
            assert_eq!(game_log.player_turn, PlayerTurn::Player1);
            assert_eq!(game_log.game_state, GameState::YetPlaying);
            assert_eq!(
                game_log.player1,
                Player {
                    my_score: Score {
                        score: 10,
                        matoi: true
                    },
                    got_score: Score {
                        score: 0,
                        matoi: false
                    },
                    name: "alice".to_string()
                }
            );
            assert_eq!(
                game_log.player2,
                Player {
                    my_score: Score {
                        score: 10,
                        matoi: true
                    },
                    got_score: Score {
                        score: 0,
                        matoi: false
                    },
                    name: "bob".to_string()
                }
            );
            assert_eq!(
                game_log.commands,
                vec![HatagenpeiCommand {
                    dice1: 4,
                    dice2: 5,
                    point: 0,
                    again: false,
                    explain: "４  ５\tごっしりはなかみ\u{3000}\u{3000}旗の移動なし"
                }]
            );
        }

        {
            let game_log = game.next().unwrap();
            assert_eq!(game_log.player_turn, PlayerTurn::Player2);
            assert_eq!(game_log.game_state, GameState::Player2Win);
            assert_eq!(
                game_log.player1,
                Player {
                    my_score: Score {
                        score: 0,
                        matoi: false
                    },
                    got_score: Score {
                        score: 0,
                        matoi: false
                    },
                    name: "alice".to_string()
                }
            );
            assert_eq!(
                game_log.player2,
                Player {
                    my_score: Score {
                        score: 10,
                        matoi: true
                    },
                    got_score: Score {
                        score: 10,
                        matoi: true
                    },
                    name: "bob".to_string()
                }
            );

            assert_eq!(game_log.commands, vec![HatagenpeiCommand { dice1: 1, dice2: 6, point: 10, again: true, explain: "１  ６\tちんろく\t\u{3000}\u{3000}\u{3000}\u{3000}中旗１本もらう\u{3000}さいころを続けて振れる" },
                                               HatagenpeiCommand { dice1: 1, dice2: 5, point: 10, again: true, explain: "１  ５\tうめがいち\u{3000}\u{3000}\t\u{3000}\u{3000}中旗１本もらう\u{3000}さいころを続けて振れる" }]);
        }
    }
}
