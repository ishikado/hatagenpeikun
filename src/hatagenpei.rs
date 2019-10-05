//!
//! 旗源平を実現するモジュール
//!


pub struct Player {
    pub score : i32,
    pub name : String
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


struct HatagenpeiCommand{
    // TODO: 実装する
}

impl Hatagenpei {
    
    /// Hatagenpei インスタンスを作成する
    pub fn new(player1 : Player, player2 : Player, first_player : PlayerTurn) -> Hatagenpei {
        return Hatagenpei{player1 : player1, player2 : player2, turn : first_player};
    }
    
    /// 1ターン進める。
    /// サイコロの振り直しが発生した場合、振り直しを行う。
    /// 戻り値で、実行ログを返す
    pub fn run(&mut self) -> Vec<String> {
        
        match self.turn {
            PlayerTurn::Player1 => {
                
            }
            PlayerTurn::Player2 => {

            }
        }
    

        return vec![];
    }

    /// サイコロを振り、行うコマンドを返す
    fn diceroll(&mut self) -> HatagenpeiCommand {
        return HatagenpeiCommand{};
    }

    /// (player1, player2) というタプルで、現在の Player ごとのスコアを取得する。
    pub fn get_players_score(&self) -> (&Player, &Player) {
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
