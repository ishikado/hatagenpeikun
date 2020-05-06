# これは何
Rust製のSlack Bot。  


# 使い方
## ビルド方法
```
$ cargo build
```

## 起動方法

NOTICE && TODO: 起動方法が変わっているので、書き直す。 
( Cargo.toml で [[bin]] を複数定義（サンプル行 && 本実行 ）しているため、それぞれ起動方法が存在する。） 

### 本起動
```
$ cargo run --bin hatagenpeikun ${slack_api_token} -l info
```

### 動作確認用起動
```
$ cargo run --bin hatagenpei_sample
```
動作確認向けに、自動的に旗源平のシミュレーションが実行される。
hatagenpeikun は、旗源平以外の機能も持っているので、テストに組み込んだほうがいいかもしれない。


# Dockerfile
dockerを利用することで、build && bot の起動、まで一気に行うことができる。

## ビルド方法（例）
```
$ docker build ./ -t $image_name --build-arg SLACK_API_TOKEN=$slack_api_token --build-arg LOG_LEVEL=$log_level
% slack_api_token と、 loglevel を指定する必要がある。
```
## 起動方法（例）
```
$ docker run --rm -it testimage
```

# heroku へのデプロイ方法
dockerを利用しているので、heroku.ymlを用意することで、デプロイを行うことができる。  
まず、以下の heroku.yml を、リポジトリルートに準備する（細かい設定に関しては、雨期のリンクも参照  https://devcenter.heroku.com/articles/build-docker-images-heroku-yml）。
```
build:
  docker:
    worker: Dockerfile
  config:
    SLACK_API_TOKEN: $slack_api_token
    LOG_LEVEL: $log_level
```

この heroku.yml を、適当にローカルブランチを切って、そのブランチにコミットする（heroku.ymlは、SLACK_API_TOKENを含んでいるため、ソースコード管理している remote にpushしないよう注意）。
```
$ git checkout -b deploy
$ git commit -m "add heroku.yml"
```

まだ heroku に application を作成していない場合、以下のコマンドで application を作成する。
```
$ heroku create $app_name
```

次に、stack に container を指定する。
```
$ heroku stack:set container
```

次に、herokuのリモートリポジトリに、push する。
```
$ git push heroku deploy:master --force
```

あとは、workerプロセスの数を1にセットすることで、bot が起動する。
```
$ heroku scale worker=1
```

workerプロセスの数を0にセットすると、botが停止する。
```
$ heroku scale worker=0
```

heroku上で動いているプロセスの状態や、ログは以下のコマンドで確認する。
```
$ heroku ps
$ heroku logs
```
