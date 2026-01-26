# src-tauri

## マイグレーションを追加

```sh
cargo sqlx migrate add <new migration name>
```

操作,命名パターン例,意味
最初の作成,create_initial_schema,最初のテーブル設計一式
テーブル作成,create_plans,plans テーブルを作成
,create_staff_groups,staff_groups テーブルを作成
カラム追加,add_memo_to_staff_members,スタッフメンバーに memo カラムを追加
,add_is_active_to_plans,プランに is_active フラグを追加
カラム変更,rename_name_to_title_in_plans,name を title に変更
,alter_sort_order_type,sort_order の型を変更
インデックス,add_index_to_plans_name,検索用にインデックスを貼る
削除,drop_old_calendar_table,古いテーブルを削除

# testing

```sh
cargo tauri dev
```
