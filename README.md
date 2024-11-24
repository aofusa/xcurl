xcURL
=====


curlを利用した負荷試験コマンド


curlコマンドを拡張し指定回数を並列して実行する

内部コマンドとしてcurlを呼び出すようにしているためcurlのオプションがそのまま利用可能  
curlがインストールされている必要がある  


使い方
-----

```sh
Usage: xcurl [OPTIONS] [-- <CURL_ARGS>...]

Arguments:
  [CURL_ARGS]...  cURL引数

Options:
  -r, --repeat <REPEAT>      curlを呼び出す回数を指定。 [default: 1]
  -t, --time <TIME>          繰り返しを行う時間を秒単位で指定します。指定された時間内で可能な限り繰り返し実行します。このオプションを使用するとき--repeatは無視されます
  -w, --wait <WAIT>          各実行間の待機時間をミリ秒単位で指定。デフォルトは待機なし。 [default: 0]
  -p, --parallel <PARALLEL>  並列で実行する数を指定。0を指定した場合repeatで指定した数を上限に可能な限り並列数を増やす。 [default: 1]
  -h, --help                 Print help
  -V, --version              Print version
```

例
```sh
# 1回だけ localhost に実行
cargo run -- -- localhost

# 3回実行を3並列で localhost に実行
cargo run -- --repeat 3 --wait 0 --parallel 3 -- localhost

# 3秒間3並列で localhost に実行
cargo run -- --time 3 --wait 0 --parallel 3 -- localhost

# 2回実行を3並列で localhost に実行
# POSTでデータも送付する
cargo run -- --repeat 2 --wait 10 --parallel 3 -- -X -d 'post data' localhost

# RUST_LOGを設定してデバッグログ出力が可能
RUST_LOG=debug cargo run -- -- localhost
```
