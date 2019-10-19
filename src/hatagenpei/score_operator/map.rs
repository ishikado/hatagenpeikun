use super::*;
use std::collections::BTreeMap;

pub struct ScoresInMap {
    score_map: BTreeMap<String, Progress>,
    winlose_map: BTreeMap<String, WinLose>,
}

impl ScoresInMap {
    pub fn new() -> ScoresInMap {
        return ScoresInMap {
            score_map: BTreeMap::new(),
            winlose_map: BTreeMap::new(),
        };
    }
}

impl ScoreOperator for ScoresInMap {
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
