//!
//! 旗源平をbotで実現するモジュール
//!

use super::game::*;
use super::score_operator::map::*;
use super::score_operator::postgre::*;
use super::score_operator::*;

const HATAGENPEI_INIT_SCORE: i32 = 29; // 小旗が両替できるように10x(x>=0) + 9 本持ちで開始すること


pub fn factor_operater( data_store : &DataStore ) ->  Box<dyn ScoreOperator> {
    let score_operator: Box<dyn ScoreOperator> = match data_store {
        DataStore::Postgre { uri } => Box::new(ScoresInPostgre::new(&uri)),
        DataStore::OnMemory => Box::new(ScoresInMap::new()),
    };
    return score_operator;
}


pub enum DataStore {
    Postgre { uri: String },
    OnMemory,
}

pub struct HatagenpeiController {
    bot_name: String,
    score_operator: Box<dyn ScoreOperator>,
}

pub struct StepResult {
    /// HatagenpeiController::step の実行ゲームログ
    pub logs: Vec<String>,
    /// ゲームが終了したかどうか
    pub is_over: bool,
    /// この step 呼び出しで、ゲームが開始したかどうか
    pub is_start: bool,
}

impl HatagenpeiController {
    pub fn new(operator : Box<dyn ScoreOperator>, bot_name: &String) -> HatagenpeiController {
        return HatagenpeiController {
            bot_name: bot_name.clone(),
            score_operator: operator,
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
            is_start: is_start,
        };
    }
}
