# 半休取得プログラム

## 説明

このソフトウェアは、正社員の半休取得を平等に割り当てることを目的としている.

## 概要



## ビルド

```sh
make BUILD=release bun-bundle
```

## ディレクトリ構成

- component-features
  // フロントエンドの画面の状態を管理するプログラム
  - wit
    Wasm Interface Typeの定義
    - deps
      jsがcomponent-features(rust)に提供する機能
    - world.wit
      component-features(rust)がmain.tsに提供する機能

- index.html

- style.css

- main.ts 
  Rust(Wasm)を呼び出しUIを表示するプログラム

- work_shift_dayoff_logic
  半休取得のアルゴリズム
