use super::*;

// TODO: このあたりの設定は https://docs.rs/config/0.9.3/config/ を使って、Settings.toml から指定できるようにしたい
const DB_HATAGENPEI_PROGRESS_KEY: &str = "hatagenpei_progress";
const DB_HATAGENPEI_WINLOSES_KEY: &str = "hatagenpei_winloses";

use postgres::{Connection, TlsMode};

pub struct ScoresInPostgre {
    postgre_uri: String,
}

impl ScoresInPostgre {
    pub fn new(postgre_uri: &String) -> ScoresInPostgre {
        // postgre に接続
        let conn = Connection::connect(&postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // progress管理テーブル作成
        let create_progress_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                    name            VARCHAR NOT NULL,
                    data            VARCHAR NOT NULL
                  )",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        conn.execute(&create_progress_table_query[..], &[])
            .expect("failed to create progress table");

        // winlose 管理テーブル作成
        let create_winlose_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                    name            VARCHAR NOT NULL,
                    data            VARCHAR NOT NULL
                  )",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        conn.execute(&create_winlose_table_query[..], &[])
            .expect("failed to create winlose table");

        return ScoresInPostgre {
            postgre_uri: postgre_uri.clone(),
        };
    }
}

impl ScoreOperation for ScoresInPostgre {
    fn get_progress(&mut self, player_name: &str) -> Option<Progress> {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");
        // 見つからなかった場合は、insert を実行する
        let select_query = format!(
            "SELECT name, data FROM {} where name = $1",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        let res = conn
            .query(&select_query[..], &[&player_name])
            .expect("failed to select query for get_progress");

        if res.len() == 0 {
            return None;
        }

        // 複数ある場合でも、1つだけ返す
        let r = res.get(0);
        let data: String = r.get(1);
        let progress = serde_json::from_str(&data[..]).expect("failed to serde_json::from_str");
        return Some(progress);
    }

    fn insert_progress(&mut self, progress: &Progress) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // すでに要素が存在している場合は、SQL update
        // そうでない場合は SQL insert を行う
        let select_query = format!(
            "SELECT name, data FROM {} where name = $1",
            DB_HATAGENPEI_PROGRESS_KEY
        );
        let res = conn
            .query(&select_query[..], &[&&progress.user.name[..]])
            .expect("failed to select query for insert_progress");

        let jsonstr = serde_json::to_string(&progress).expect("failed to serde_json::to_string");
        if res.len() == 0 {
            // insert
            let insert_query = format!(
                "INSERT INTO {} (name, data) VALUES ($1, $2)",
                DB_HATAGENPEI_PROGRESS_KEY
            );
            conn.execute(&insert_query[..], &[&&progress.user.name[..], &jsonstr])
                .expect("failed to insert query for insert_progress");
        } else {
            // update
            let update_query = format!(
                "UPDATE {} SET data = $1 WHERE name = $2",
                DB_HATAGENPEI_PROGRESS_KEY
            );
            conn.execute(&update_query[..], &[&jsonstr, &&progress.user.name[..]])
                .expect("failed to update query for insert_progress");
        }
        return true;
    }

    fn delete_progress(&mut self, player_name: &str) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");
        let delete_query = format!("DELETE FROM {} where name = $1", DB_HATAGENPEI_PROGRESS_KEY);
        conn.execute(&delete_query[..], &[&player_name])
            .expect("failed to delete query for delete_progress");

        return true;
    }
    fn update_winloses(&mut self, player_name: &str, is_player_win: bool) -> bool {
        // postgre に接続
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // 勝敗を取得
        let select_query = format!(
            "SELECT name, data, data FROM {} where name = $1",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        let res = conn
            .query(&select_query[..], &[&player_name])
            .expect("failed to select query for update_winloses");

        let mut win_lose = if res.len() == 0 {
            WinLose::new(0, 0, player_name)
        } else {
            let r = res.get(0);
            let data: String = r.get(1);
            serde_json::from_str(&data[..]).expect("failed to serde_json::from_str")
        };

        if is_player_win {
            win_lose.win += 1;
        } else {
            win_lose.lose += 1;
        }

        let s = serde_json::to_string(&win_lose).expect("failed to serde_json::to_string");

        // insert
        if res.len() == 0 {
            let insert_query = format!(
                "INSERT INTO {} (name, data) VALUES ($1, $2)",
                DB_HATAGENPEI_WINLOSES_KEY
            );
            conn.execute(&insert_query[..], &[&player_name, &s])
                .expect("failed to insert query for update_winloses");
        }
        // update
        else {
            let update_query = format!(
                "UPDATE {} SET data = $1 WHERE name = $2",
                DB_HATAGENPEI_WINLOSES_KEY
            );
            conn.execute(&update_query[..], &[&s, &player_name])
                .expect("failed to update query for update_winloses");
        }

        return true;
    }

    fn get_win_loses(&self) -> Vec<WinLose> {
        let mut res = vec![];
        let conn = Connection::connect(&self.postgre_uri[..], TlsMode::None)
            .expect("failed to connect postgres");

        // 勝敗を取得
        let select_query = format!(
            "SELECT name, data, data FROM {}",
            DB_HATAGENPEI_WINLOSES_KEY
        );
        let query_result = conn
            .query(&select_query[..], &[])
            .expect("failed to select query for get_win_loses");

        for row in &query_result {
            let data: String = row.get(1);
            let win_lose = serde_json::from_str(&data[..]).expect("failed to serde_json::from_str");
            res.push(win_lose);
        }
        return res;
    }
}
