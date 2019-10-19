//!
//! Hatagenpeicontroller実行のサンプルコード
//!

fn main() {
    // controller を動作させ、ちゃんと状態が保存されているか見る
    use hatagenpeikun::hatagenpei::controller::*;

    let mut ins = HatagenpeiController::new(
        factor_operater(&DataStore::OnMemory),
        &"hatagenpeikun".to_string(),
    );
    for _ in 0..3 {
        loop {
            let res = ins.step(&"rust".to_string());
            for l in &res.logs {
                println!("{:?}", l);
            }
            if res.is_over {
                break;
            }
        }
    }

    println!("{:?}", ins.get_win_loses());
}
