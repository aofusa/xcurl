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
  -p, --parallel <PARALLEL>  並列で実行する数を指定。0の場合可能な限り並列数を増やす。 [default: 1]
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
cargo run -- --repeat 2 --wait 10 --parallel 3 -- -X POST -d 'post data' localhost

# RUST_LOGを設定してデバッグログ出力が可能
RUST_LOG=debug cargo run -- -- localhost
```

実行後は以下のような簡易統計情報が表示される。単位はミリ秒時間
```json
{"mean_time":65,"max_time":98,"min_time":19,"variance_time":11,"quartile_25":56,"quartile_75":75,"status_count":{"200":100},"error_count":0}
```

Dockerを使い多重起動する例
```sh
# 事前にstaticリンクしたバイナリを準備
docker run --rm -it -v $(pwd):/home/rust/src messense/rust-musl-cross:i686-musl cargo build --release

# 複数コンテナからxcurlを実行
for _i in $(seq 1 5); do
    docker run --rm --volume $(pwd)/target/i686-unknown-linux-musl/release:/app:ro -d alpine/curl sh -c "/app/xcurl --time 10 --parallel 100 -- localhost" &
done
```


各環境向けビルド手順
-----

以下のビルドツールを利用
- linux・mac：zigbuild
- windows：cargo-xwin
- musl：musl-closs

[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)  
[cargo-xwin](https://github.com/rust-cross/cargo-xwin)  
[rust-musl-cross](https://github.com/rust-cross/rust-musl-cross)


参考

Rustは2022年公開の1.64より最低要件を glibc >= 2.17, kernel >= 3.2 にしている
[Rust Blog](https://blog.rust-lang.org/2022/08/01/Increasing-glibc-kernel-requirements.html)

そのためglibc のバージョン 2.17 以前の環境で使う場合はmuslでstatic linkしたものを使用する


### ビルドコマンド

コマンドを実行する場合はdockerが必要

```sh
docker run --rm -it -v $(pwd):/io -w /io messense/cargo-zigbuild cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.17
docker run --rm -it -v $(pwd):/io -w /io messense/cargo-zigbuild cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.17
docker run --rm -it -v $(pwd):/io -w /io messense/cargo-zigbuild cargo zigbuild --release --target universal2-apple-darwin
docker run --rm -it -v $(pwd):/io -w /io messense/cargo-xwin cargo xwin build --release --target x86_64-pc-windows-msvc
docker run --rm -it -v $(pwd):/io -w /io messense/cargo-xwin cargo xwin build --release --target aarch64-pc-windows-msvc
docker run --rm -it -v $(pwd):/home/rust/src messense/rust-musl-cross:i686-musl cargo build --release
```
