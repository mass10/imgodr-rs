# imgodr-rs

画像ファイルの EXIF 0x9003: 撮影日時 を読み取り、ファイル名を `2014年01月10日 23時06分42秒.jpg` の形式にリネームするコマンドラインユーティリティです。

# Getting Started

```
cargo install --git https://github.com/yourname/imgodr-rs.git
```

# 使い方

```
imgodr-rs <ディレクトリ or ファイル>...
```

ディレクトリを指定すると再帰的に処理します。
