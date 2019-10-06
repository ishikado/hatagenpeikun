//!
//! 旗源平を実現するモジュール
//!

extern crate rand;
use rand::prelude::*;
use std::mem;
use std::string::ToString;


pub struct Score {
    pub score : i32
}

impl ToString for Score {
    fn to_string(&self) -> String {
        let obata = self.score / 100;
        let chubata = (self.score % 100) / 10;
        let kobata = (self.score % 100) % 10;
        return format!("大旗 : {} 本、中旗 : {} 本、小旗 : {} 本", obata, chubata, kobata).to_string();
    }
}


#[derive(Clone)]
pub struct Player {
    pub score : i32,
    pub name : String
}

impl Player {
    pub fn new(name : String) -> Player {
        return Player{score : 0, name : name};
    }
}

pub enum PlayerTurn {
    Player1,
    Player2
}

pub enum VictoryOrDefat {
    Player1Win,
    Player2Win,
    YetPlaying
}

pub struct Hatagenpei {
    player1 : Player,
    player2 : Player,
    turn : PlayerTurn
}

pub enum HatagenPeiError {
    Unexpected
}


#[derive(Clone)]
struct HatagenpeiCommand{
    dice1 : u8,
    dice2 : u8,
    point : i32,   // + なら もらう、- ならあげる
    again : bool , // true ならもういちどサイコロを 
    explain : &'static str, // 説明文
}


const hatagenpeicommands : [HatagenpeiCommand ; 21]
    = [
        HatagenpeiCommand{dice1 : 1, dice2 : 1, point : 2, again : true,  explain : "１  １	ちんちんかもかも　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 2, dice2 : 2, point : 2, again : true,  explain : "２  ２	にゃあにゃあ	　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 3, dice2 : 3, point : 2, again : true,  explain : "３  ３	さざなみ	　　　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 4, dice2 : 4, point : 2, again : true,  explain : "４  ４	しゅうじゅう	　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 5, dice2 : 5, point : 2, again : true,  explain : "５  ５	ごんご	　　　　　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 6, dice2 : 6, point : 2, again : true,  explain : "６  ６	じょうろく	　　　　小旗２本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 1, dice2 : 2, point : 1, again : false, explain : "１  ２	ちんに	　　　　　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 1, dice2 : 3, point : 1, again : false, explain : "１  ３	ちんさん　　	　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 1, dice2 : 4, point : 1, again : false, explain : "１  ４	ちんし	　　　　　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 1, dice2 : 5, point : 10, again : true, explain : "１  ５	うめがいち　　	　　中旗１本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 1, dice2 : 6, point : 10, again : true, explain : "１  ６	ちんろく	　　　　中旗１本もらう　さいころを続けて振れる"},
        HatagenpeiCommand{dice1 : 2, dice2 : 3, point : 0, again : false, explain : "２  ３	にさまのかんかんど	旗の移動なし"},
        HatagenpeiCommand{dice1 : 2, dice2 : 4, point : -10, again : false, explain : "２  ４	しのに	　　　　　　中旗１本返す"},
        HatagenpeiCommand{dice1 : 2, dice2 : 5, point : 0, again : false, explain : "２  ５	ごにごに	　　　　旗の移動なし"},
        HatagenpeiCommand{dice1 : 2, dice2 : 6, point : 1, again : false, explain : "２  ６	ろくに	　　　　　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 3, dice2 : 4, point : 0, again : false, explain : "３  ４	しさまのかんかんど　旗の移動なし"},
        HatagenpeiCommand{dice1 : 3, dice2 : 5, point : 0, again : false, explain : "３  ５	ごさまのかんかんど　旗の移動なし"},
        HatagenpeiCommand{dice1 : 3, dice2 : 6, point : 1, again : false, explain : "３  ６	ろくさん	　　　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 4, dice2 : 5, point : 0, again : false, explain : "４  ５	ごっしりはなかみ　　旗の移動なし"},
        HatagenpeiCommand{dice1 : 4, dice2 : 6, point : 1, again : false, explain : "４  ６	しろく	　　　　　　小旗１本もらう"},
        HatagenpeiCommand{dice1 : 5, dice2 : 6, point : 1, again : false, explain : "５  ６	ごろく	　　　　　　小旗１本もらう"}
    ];

impl Hatagenpei {
    
    /// Hatagenpei インスタンスを作成する
    pub fn new(player1 : Player, player2 : Player, first_player : PlayerTurn) -> Hatagenpei {
        return Hatagenpei{player1 : player1, player2 : player2, turn : first_player};
    }
    
    /// 1ターン進める。
    /// サイコロの振り直しが発生した場合、振り直しを行う。
    /// 戻り値で、実行ログを返す
    pub fn next(&mut self) -> Vec<String> {
        let res = vec![];
        
        // まだプレイ中でなければならない
        match self.get_victory_or_defeat() {
            Ok(VictoryOrDefat::YetPlaying) => {}
            _ => {
                return res;
            }
        }

        let mut play_player;
        let mut wait_player;        
        let next_turn;

        match self.turn {
            PlayerTurn::Player1 => {
                play_player = &mut self.player1;
                wait_player = &mut self.player2;
                next_turn = PlayerTurn::Player2;
            }
            PlayerTurn::Player2 => {
                play_player = &mut self.player2;
                wait_player = &mut self.player1;
                next_turn = PlayerTurn::Player1;
            }
        }

        
        loop {
            match self.get_victory_or_defeat() {
                Ok(VictoryOrDefat::YetPlaying) => {
                    // まだプレイ中の場合のみダイスを振る
                    let cmd = Self::diceroll();
                    Play_player.score += cmd.point;
                    

                    // もう一度振れないなら終了
                    if !cmd.again {
                        break;
                    }
                }
                _ => {
                    return vec![];
                }
            }

            
        }
        self.turn = next_turn;
        return res;
    }

    /// サイコロを振り、行うコマンドを返す
    fn diceroll() -> HatagenpeiCommand {
        // 乱数でサイコロの目を決める
        let mut d1 = (rand::random::<u8>() % 6) + 1;
        let mut d2 = (rand::random::<u8>() % 6) + 1;
        if d1 > d2 {
            mem::swap(&mut d1, &mut d2);
        }

        let cmd = hatagenpeicommands.iter().find(|cmd| {
            return cmd.dice1 == d1 && cmd.dice2 == d2;
        }).unwrap();

        return cmd.clone();
    }

    /// (player1, player2) というタプルで、現在の Player ごとのスコアを取得する。
    pub fn get_score(&self) -> (&Player, &Player) {
        return (&self.player1, &self.player2);
    }

    pub fn get_victory_or_defeat(&self) ->  Result<VictoryOrDefat, HatagenPeiError> {
        if self.player1.score == 0 && self.player2.score > 0 {
            return Ok(VictoryOrDefat::Player2Win);
        }
        else if self.player1.score > 0 && self.player2.score == 0 {
            return Ok(VictoryOrDefat::Player1Win);
        }
        else if self.player1.score > 0 && self.player2.score > 0 {
            return Ok(VictoryOrDefat::YetPlaying);
        }
        else{
            return Err(HatagenPeiError::Unexpected);
        }
    }
    
}

