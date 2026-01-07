
JS関数名,操作内容,Rustメソッド名案,引数
addNewGroup,グループ追加,add_group,なし
removeGroup,グループ削除,remove_group,index: u32
updateGroupName,名前変更,update_group_name,"index: u32, name: string"
addSlot,スロット追加,add_slot,group_idx: u32
removeSlot,スロット削除,remove_slot,"group_idx: u32, slot_idx: u32"
updateSlotMemo,メモ更新,update_slot_memo,"group_idx: u32, slot_idx: u32, memo: string"
addNewRule,ルール追加,add_rule,なし
removeRule,ルール削除,remove_rule,index: u32
updateRuleName,ルール名変更,update_rule_name,"index: u32, name: string"
confirmAssignment,ルールにID追加,add_rule_assignment,"rule_idx: u32, day: string, shift: string, id: string"
removeAssignment,ルールからID削除,remove_rule_assignment,"rule_idx: u32, day: string, shift: string, array_idx: u32"
prev/next-btn,月移動,change_month,delta: i32 (+1 or -1)
generateSchedule,生成ロジック,generate_schedule,rule_idx: u32
