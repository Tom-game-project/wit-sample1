import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import type { 
    Plan, PlanConfig, StaffGroupWithMembers, WeeklyRuleWithAssignments, 
    ShiftCalendarManager, WeekStatus, RuleAssignment, MonthlyShiftResult
} from "./types";

/* ==========================================================================
   STATE
   ========================================================================== */
let currentPlanId: number | null = null;
let currentConfig: PlanConfig | null = null;
let currentYear = new Date().getFullYear();
let currentMonth = new Date().getMonth();

// 編集中のスキップ状態を保持するマップ
// Key: "YYYY-MM-DD" (週の月曜日の日付文字列)
// Value: true = Skip, false = Active
let pendingSkips: Record<string, boolean> = {};
// 状態を表す型定義
type WeekState = 'pending_active' | 'pending_skip' | 'fixed_active' | 'fixed_skip';


// ==========================================
// UTILITIES
// ==========================================

// グループごとの色パレット
const GROUP_COLORS = [
    '#e67e22', // Orange (A)
    '#27ae60', // Green (B)
    '#2980b9', // Blue (C)
    '#8e44ad', // Purple (D)
    '#c0392b', // Red (E)
    '#16a085', // Teal (F)
    '#d35400', // Pumpkin (G)
    '#2c3e50', // Midnight (H)
];

function getGroupColor(index: number): string {
    return GROUP_COLORS[index % GROUP_COLORS.length];
}

function getGroupPrefix(index: number): string {
    // 0 -> A, 1 -> B ...
    return String.fromCharCode(65 + index);
}

/* ==========================================================================
   INIT & PLAN
   ========================================================================== */
window.addEventListener('DOMContentLoaded', async () => {
    setupEventListeners();
    await loadPlanList();
});

async function loadPlanList() {
    try {
        const plans = await invoke<Plan[]>("list_all_plans");
        const select = document.getElementById('plan-select') as HTMLSelectElement;
        select.innerHTML = '<option value="" disabled selected>Select Plan...</option>';
        plans.forEach(plan => {
            const opt = document.createElement('option');
            opt.value = plan.id.toString();
            opt.textContent = plan.name;
            select.appendChild(opt);
        });
        
        // 直近のPlanがあれば自動選択するロジックをここに入れる
    } catch (e) {
        console.error("Failed to list plans", e);
    }
}

async function handlePlanChange(planId: number) {
    currentPlanId = planId;
    console.log("Plan Changed:", planId);
    await reloadConfig();
    await renderCalendarView();
}

async function createNewPlan() {
    const name = prompt("Enter new plan name:");
    if (!name) return;
    try {
        const newId = await invoke<number>("create_new_plan", { name });
        await loadPlanList();
        (document.getElementById('plan-select') as HTMLSelectElement).value = newId.toString();
        handlePlanChange(newId);
    } catch (e) {
        alert("Failed to create plan: " + e);
    }
}

/* ==========================================================================
   CONFIG VIEW
   ========================================================================== */
async function reloadConfig() {
    if (!currentPlanId) return;
    try {
        currentConfig = await invoke<PlanConfig>("get_plan_config", { planId: currentPlanId });
        renderConfigUI(currentConfig);
    } catch (e) {
        console.error("Failed to load config", e);
    }
}

function renderConfigUI(config: PlanConfig) {
    renderGroups(config.groups);
    renderRules(config.rules);
    const jsonEl = document.getElementById('json-output');
    if (jsonEl) jsonEl.textContent = JSON.stringify(config, null, 2);
}

// Groups Logic
function renderGroups(groups: StaffGroupWithMembers[]) {
    const container = document.getElementById('staff-groups-container');
    if (!container) return;
    container.innerHTML = '';

    groups.forEach((g, index) => {
        // 自動割り当ての色とプレフィックスを取得
        const color = getGroupColor(index);
        const prefix = getGroupPrefix(index);

        const div = document.createElement('div');
        div.className = 'group-card';
        // 見た目のスタイル調整
        div.style.background = '#fff';
        div.style.borderRadius = '6px';
        div.style.boxShadow = '0 2px 5px rgba(0,0,0,0.05)';
        div.style.marginBottom = '15px';
        div.style.overflow = 'hidden';
        // ★ 左側に色付きのラインを入れる
        div.style.borderLeft = `5px solid ${color}`;

        div.innerHTML = `
            <div style="padding: 10px; background: #f8f9fa; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center;">
                <div style="display:flex; align-items:center; gap:8px;">
                    <span style="background:${color}; color:white; font-weight:bold; padding:2px 8px; border-radius:4px; font-size:0.9em;">
                        ${prefix}
                    </span>
                    <strong>${g.group.name}</strong>
                </div>
                <div>
                    <button class="btn-sm btn-outline" onclick="window.updateGroupName(${g.group.id})">Rename</button>
                    <button class="btn-sm btn-danger" onclick="window.removeGroup(${g.group.id})">Del</button>
                </div>
            </div>
            <div class="members-list" style="padding: 10px;"></div>
        `;

        const list = div.querySelector('.members-list')!;

        if (g.members.length === 0) {
            list.innerHTML = '<div style="color:#aaa; font-size:0.9em; font-style:italic;">No members yet.</div>';
        }

        g.members.forEach(m => {
            const mDiv = document.createElement('div');
            mDiv.style.display = 'flex';
            mDiv.style.justifyContent = 'space-between';
            mDiv.style.alignItems = 'center';
            mDiv.style.padding = '5px 0';
            mDiv.style.borderBottom = '1px solid #f0f0f0';

            mDiv.innerHTML = `
                <div style="display:flex; align-items:center; gap:5px;">
                    <span style="color:#888; font-size:0.8em; width:20px;">#${m.sort_order}</span>
                    <span>${m.name}</span>
                </div>
                <div>
                    <button class="btn-sm btn-outline" style="font-size:0.7em; margin-right:5px;" onclick="window.updateMemberName(${m.id})">Edit</button>
                    <button class="btn-sm btn-outline-danger" style="font-size:0.7em;" onclick="window.removeMember(${m.id})">x</button>
                </div>
            `;
            list.appendChild(mDiv);
        });

        // Footer (Add Button)
        const footer = document.createElement('div');
        footer.style.padding = '0 10px 10px 10px';

        const addBtn = document.createElement('button');
        addBtn.className = "btn-sm btn-outline";
        addBtn.style.width = "100%";
        addBtn.style.borderStyle = "dashed";
        addBtn.textContent = "+ Add Member";
        addBtn.onclick = () => addMember(g.group.id);

        footer.appendChild(addBtn);
        div.appendChild(footer);

        container.appendChild(div);
    });
}

// Rules Logic
// // renderRules関数内の、Assignments表示ループと追加ボタン部分を修正
function renderRules(rules: WeeklyRuleWithAssignments[]) {
    const container = document.getElementById('rules-container');
    if (!container) return;
    container.innerHTML = '';

    rules.forEach((r, rIdx) => {
        const div = document.createElement('div');
        div.className = 'rule-card';
        div.style.border = '1px solid #ccc';
        div.style.padding = '15px';
        div.style.marginBottom = '15px';
        div.style.background = '#fff';
        div.style.borderRadius = '8px';

        div.innerHTML = `
            <div style="display:flex; justify-content:space-between; align-items:center; border-bottom:1px solid #eee; padding-bottom:10px; margin-bottom:10px;">
                <div style="display:flex; align-items:center; gap:10px;">
                    <span style="background:#333; color:white; padding:2px 6px; border-radius:4px; font-size:0.8em;">#${rIdx + 1}</span>
                    <strong style="font-size:1.1em;">${r.rule.name}</strong>
                </div>
                <div>
                    <button class="btn-sm btn-outline" onclick="window.updateRuleName(${r.rule.id})">Rename</button>
                    <button class="btn-sm btn-danger" onclick="window.removeRule(${r.rule.id})">Del</button>
                </div>
            </div>
            
            <div class="assignments-grid" style="overflow-x:auto;">
                <table style="width:100%; border-collapse: collapse; font-size:0.9em;">
                    <thead>
                        <tr style="background:#f9f9f9; text-align:left; border-bottom:2px solid #eee;">
                            <th style="padding:8px;">Time</th>
                            ${['Mon','Tue','Wed','Thu','Fri','Sat','Sun'].map(d => `<th style="padding:8px; min-width:80px;">${d}</th>`).join('')}
                        </tr>
                    </thead>
                    <tbody id="rule-table-body-${r.rule.id}"></tbody>
                </table>
            </div>
        `;
        
        container.appendChild(div);

        const tbody = document.getElementById(`rule-table-body-${r.rule.id}`)!;
        [0, 1].forEach(shiftTime => {
            const tr = document.createElement('tr');
            tr.style.borderTop = '1px solid #eee';
            
            const timeLabel = document.createElement('td');
            timeLabel.textContent = shiftTime === 0 ? "AM" : "PM";
            timeLabel.style.fontWeight = "bold";
            timeLabel.style.padding = "8px";
            timeLabel.style.color = shiftTime === 0 ? "#e67e22" : "#2980b9";
            tr.appendChild(timeLabel);

            for(let weekday=0; weekday<7; weekday++) {
                const td = document.createElement('td');
                td.style.padding = "5px";
                td.style.verticalAlign = "top";
                td.style.borderLeft = "1px solid #f5f5f5";

                // このセルに該当するアサインメントを抽出
                const assigns = r.assignments.filter(a => a.weekday === weekday && a.shift_time_type === shiftTime);
                
                assigns.forEach(a => {
                    // グループ情報を検索して色やプレフィックスを決定
                    const groupIndex = currentConfig?.groups.findIndex(g => g.group.id === a.target_group_id) ?? -1;
                    const groupData = groupIndex >= 0 ? currentConfig!.groups[groupIndex] : null;
                    
                    const color = groupIndex >= 0 ? getGroupColor(groupIndex) : '#999';
                    const prefix = groupIndex >= 0 ? getGroupPrefix(groupIndex) : '?';
                    
                    // ツールチップ用に本来の名前を取得
                    const memberName = groupData?.members[a.target_member_index]?.name || "Unknown";

                    // ★ 色付きチップの生成
                    const chip = document.createElement('div');
                    
                    // スタイル設定: 左端を太い色線にするデザイン
                    chip.style.border = `1px solid ${color}`;
                    chip.style.borderLeft = `5px solid ${color}`; 
                    chip.style.background = '#fcfcfc';
                    chip.style.color = '#333';
                    chip.style.padding = '2px 6px';
                    chip.style.borderRadius = '3px';
                    chip.style.marginBottom = '4px';
                    chip.style.fontSize = '0.9em'; // 少し大きく
                    chip.style.fontWeight = 'bold';
                    chip.style.cursor = 'pointer';
                    chip.style.whiteSpace = 'nowrap';
                    chip.style.boxShadow = '0 1px 2px rgba(0,0,0,0.1)';
                    chip.style.display = 'inline-block';
                    chip.style.marginRight = '4px';
                    
                    // ★ 表示内容: "A-0" のような形式
                    chip.textContent = `${prefix}-${a.target_member_index}`;
                    
                    // ホバー時に詳細（グループ名: 名前）を表示
                    chip.title = `${groupData?.group.name || 'Unknown'}: ${memberName}`;
                    
                    // クリックで削除
                    chip.onclick = (e) => {
                        e.stopPropagation();
                        if(confirm(`Remove assignment ${prefix}-${a.target_member_index} (${memberName})?`)) {
                            removeAssignment(a.id);
                        }
                    };
                    td.appendChild(chip);
                });

                // 追加ボタン (+)
                const addBtn = document.createElement('button');
                addBtn.textContent = "+";
                addBtn.className = "btn-sm btn-outline-light";
                addBtn.style.color = "#ccc";
                addBtn.style.fontSize = "0.7em";
                addBtn.style.display = "block";
                addBtn.style.width = "100%";
                addBtn.style.marginTop = "5px";
                addBtn.onclick = () => openAssignmentModal(r.rule.id, weekday, shiftTime);
                td.appendChild(addBtn);

                tr.appendChild(td);
            }
            tbody.appendChild(tr);
        });
    });
}


function openAssignmentModal(ruleId: number, weekday: number, shiftTime: number) {
    if (!currentConfig) return;

    const modal = document.getElementById('modal');
    const modalBody = document.getElementById('modal-body');
    const modalTitle = document.getElementById('modal-title');

    if (!modal || !modalBody || !modalTitle) return;

    // タイトル設定
    const dayName = ['Mon','Tue','Wed','Thu','Fri','Sat','Sun'][weekday];
    const timeName = shiftTime === 0 ? "Morning" : "Afternoon";
    modalTitle.textContent = `Assign to ${dayName} - ${timeName}`

    // コンテンツ生成
    modalBody.innerHTML = '';

    if (currentConfig.groups.length === 0) {
        modalBody.innerHTML = '<p>No staff groups defined yet.</p>';
    }

    // ★修正: indexを受け取るように変更
    currentConfig.groups.forEach((g, index) => {
        // ★追加: 色とプレフィックスを取得
        const color = getGroupColor(index);
        const prefix = getGroupPrefix(index);

        const groupDiv = document.createElement('div');
        groupDiv.style.marginBottom = '15px';

        const header = document.createElement('div');
        header.style.fontWeight = 'bold';
        
        // ★修正: 色を適用
        header.style.color = color; 
        header.style.borderBottom = `2px solid ${color}`; // ボーダーも色付きに
        header.style.marginBottom = '5px';
        
        // ★修正: プレフィックスと名前を表示
        header.innerHTML = `<span style="font-weight:900; margin-right:5px;">${prefix}</span> ${g.group.name}`;
        
        groupDiv.appendChild(header);

        const grid = document.createElement('div');
        grid.style.display = 'grid';
        grid.style.gridTemplateColumns = 'repeat(auto-fill, minmax(100px, 1fr))';
        grid.style.gap = '8px';

        // メンバー一覧ボタン
        g.members.forEach((m, mIndex) => { // 変数名を mIndex に変更
            const btn = document.createElement('button');
            btn.className = 'btn btn-outline-light';
            btn.style.color = '#333';
            
            // ★修正: 左端に色付きのボーダーを追加して視認性を向上
            btn.style.border = '1px solid #ddd';
            btn.style.borderLeft = `4px solid ${color}`;

            btn.style.padding = '8px';
            btn.style.textAlign = 'center';
            btn.style.cursor = 'pointer';
            btn.textContent = `#${mIndex + 1}`;

            btn.onclick = async () => {
                // アサイン実行
                await addAssignment(ruleId, weekday, shiftTime, g.group.id, mIndex);
                closeModal();
            };

            grid.appendChild(btn);
        });

        groupDiv.appendChild(grid);
        modalBody.appendChild(groupDiv);
    });

    // 表示
    modal.style.display = 'flex';
}

function closeModal() {
    const modal = document.getElementById('modal');
    if (modal) modal.style.display = 'none';
}


// Actions
async function addNewGroup() {
    if (!currentPlanId) return;
    await invoke("add_staff_group", { planId: currentPlanId, name: "New Group" });
    reloadConfig();
}

async function removeGroup(groupId: number) {
    if(!confirm("Delete group?")) return;
    await invoke("delete_staff_group", { groupId });
    reloadConfig();
}

async function updateGroupName(groupId: number) {
    const name = prompt("New name:");
    if(name) { await invoke("update_group_name", { groupId, name }); reloadConfig(); }
}

async function addMember(groupId: number) {
    await invoke("add_staff_member", { groupId, name: "New Member" });
    reloadConfig();
}

async function removeMember(memberId: number) {
    await invoke("delete_staff_member", { memberId });
    reloadConfig();
}

async function addNewRule() {
    if (!currentPlanId) return;
    await invoke("add_weekly_rule", { planId: currentPlanId, name: "New Rule" });
    reloadConfig();
}

async function removeRule(ruleId: number) {
    if(!confirm("Delete rule?")) return;
    await invoke("delete_weekly_rule", { ruleId });
    reloadConfig();
}

async function addAssignment(ruleId: number, weekday: number, shiftTime: number, groupId: number, memberIndex: number) {
    await invoke("add_rule_assignment", { ruleId, weekday, shiftTime, groupId, memberIndex });
    reloadConfig();
}

async function removeAssignment(assignmentId: number) {
    // 誤操作防止の確認
    if (!confirm("Remove this assignment?")) return; // 確認が煩わしい場合はコメントアウトしてください

    try {
        // Rustコマンド呼び出し
        await invoke("delete_assignment", { assignmentId });
        
        // 画面更新
        await reloadConfig();
    } catch (e) {
        console.error("Failed to remove assignment:", e);
        alert(`Failed to remove assignment: ${e}`);
    }
}

async function updateMemberName(memberId: number) {
    // 現在の名前を取得できれば良いですが、今回は単純に空欄プロンプトから始めます
    // (UX向上のため、本来は現在の名前を初期値に入れたいところですが、
    //  memberIdから逆引きするのが少し手間なので省略します。必要ならfindしてください)
    
    const newName = prompt("Enter new name:");
    if (newName && newName.trim() !== "") {
        try {
            await invoke("update_member_name", { memberId, name: newName.trim() });
            await reloadConfig();
        } catch (e) {
            alert(`Failed to update member name: ${e}`);
        }
    }
}

// Generateボタン: UI上のスキップ設定を集めてバックエンドへ送る
async function handleGenerate() {
    if (!currentPlanId) {
        alert("Please select a plan first.");
        return;
    }

    // 1. UI上の全スイッチの状態を収集する
    //    pendingSkips だけだと「変更していないスイッチ」が漏れる可能性があるため、
    //    DOM要素(checkbox)から直接「今の見た目通り」の状態を取得するのが最も確実です。
    
    const mount = document.getElementById('calendar-mount');
    if (!mount) return;

    const checkboxes = mount.querySelectorAll('input[type="checkbox"]');
    const skipFlags: boolean[] = [];
    
    // NodeListを配列に変換して処理
    checkboxes.forEach((cb) => {
        // disableされている(FIXED)週は、生成対象外とみなすか、
        // あるいは「上書き生成」の実装方針によります。
        // ★今回の実装方針: 「FIXEDより後の週（Pending）」のみを対象として送る
        const input = cb as HTMLInputElement;
        
        if (!input.disabled) {
            skipFlags.push(input.checked);
        }
    });

    if (skipFlags.length === 0) {
        alert("No new (pending) weeks to generate.");
        return;
    }

    console.log("Sending skips to Rust:", skipFlags);

    try {
        // 2. Rustコマンド呼び出し
        await invoke("generate_and_save_shift", { 
            planId: currentPlanId, 
            skips: skipFlags 
        });
        
        // 3. 成功したらローカルのpending状態をクリアして再描画
        pendingSkips = {}; // クリア
        await renderCalendarView(); // 最新の保存状態(FIXED)として再描画されるはず
        
        alert("Schedule Generated & Saved!");
        
    } catch (e) {
        console.error(e);
        alert(`Generate failed: ${e}`);
    }
}

/* ==========================================================================
   CALENDAR VIEW
   ========================================================================== */

async function renderCalendarView() {
    if (!currentPlanId) return;

    const label = document.getElementById('current-month-label');
    if (label) {
        const date = new Date(currentYear, currentMonth, 1);
        label.textContent = date.toLocaleDateString('ja-JP', { year: 'numeric', month: 'long' });
    }

    const mount = document.getElementById('calendar-mount');
    if (!mount) return;
    mount.innerHTML = '<div style="padding:20px; text-align:center;">Loading...</div>';

    // 1. カレンダーの日付構造を計算
    const weeksData = calculateCalendarDates(currentYear, currentMonth);

    // 1. Rustから「確定シフトデータ」を取得
    let shiftData: MonthlyShiftResult = { weeks: [] };
    try {
        shiftData = await invoke<MonthlyShiftResult>("derive_monthly_shift", {
            planId: currentPlanId,
            targetYear: currentYear,
            targetMonth: currentMonth
        });
        console.log("shiftData", shiftData);
    } catch (e) {
        console.error("Failed to derive shifts:", e);
    }

    // 2. DBから保存済みの状態を取得 (Fixed判定用)
    let savedTimeline: WeekStatus[] = [];
    try {
        const savedManager = await invoke<ShiftCalendarManager>("get_calendar_state", { planId: currentPlanId });
        if (savedManager) {
            savedTimeline = savedManager.timeline;
        }
    } catch (e) {
        // 保存データがない場合は空の配列として扱う
    }

    mount.innerHTML = '';

    weeksData.forEach((week, i) => {
        // 週を識別するキー (月曜日の日付文字列)
        const weekKey = week.days[0].toISOString().split('T')[0];

        // --- ステータス決定ロジック ---
        let state: WeekState = 'pending_active'; // デフォルト

        // DBの保存データと照合 (簡易的に配列インデックス i を使用)
        // ※ 本格運用では abs_week の計算と照合が必要ですが、現状はこれで動作します
        const savedStatus = savedTimeline[i];

        if (savedStatus) {
            // ■ 確定済み (DBにデータあり)
            // RustのEnumはTagged Union { type: "active", ... } | { type: "skipped", ... } で返る
            if (savedStatus.type === 'skipped') {
                state = 'fixed_skip';
            } else {
                state = 'fixed_active';
            }
        } else {
            // ■ 未確定 (DBにデータなし -> UIの操作履歴を参照)
            if (pendingSkips[weekKey] === true) {
                state = 'pending_skip';
            } else {
                state = 'pending_active';
            }
        }

        // --- DOM生成 ---
        const row = document.createElement('div');
        row.className = 'cal-week-row'; // CSS: .cal-week-row

        // [左カラム] コントロール (スイッチ & ラベル)
        const controlCell = document.createElement('div');
        controlCell.className = 'cal-cell-control'; // CSS: .cal-cell-control

        // スイッチ部分
        const switchLabel = document.createElement('label');
        switchLabel.className = 'switch'; // CSS: .switch

        const input = document.createElement('input');
        input.type = 'checkbox';

        // 状態に応じた Input の設定
        switch (state) {
            case 'pending_active':
                input.checked = false;
                input.disabled = false;
                break;
            case 'pending_skip':
                input.checked = true;
                input.disabled = false;
                break;
            case 'fixed_active':
                input.checked = false;
                input.disabled = true; // 変更不可
                break;
            case 'fixed_skip':
                input.checked = true;
                input.disabled = true; // 変更不可
                break;
        }

        // イベント: 未確定(Pending)の場合のみ状態を更新
        input.onchange = (e) => {
            const isChecked = (e.target as HTMLInputElement).checked;
            pendingSkips[weekKey] = isChecked;

            // ラベルの動的更新
            const textEl = controlCell.querySelector('.status-text') as HTMLElement;
            if (textEl) {
                textEl.textContent = isChecked ? "SKIP" : "ACTIVE";
                textEl.className = `status-text ${isChecked ? 'text-skip' : 'text-active'}`;
            }
        };

        const slider = document.createElement('span');
        // CSSクラスを動的に付与して色を変える (例: slider fixed-active)
        slider.className = `slider ${state.replace('_', '-')}`;

        switchLabel.appendChild(input);
        switchLabel.appendChild(slider);
        controlCell.appendChild(switchLabel);

        // テキストラベル部分
        const statusText = document.createElement('span');
        statusText.classList.add('status-text'); // 共通クラス
        
        let labelText = "";
        let labelColorClass = "";

        switch (state) {
            case 'pending_active': 
                labelText = "ACTIVE"; 
                labelColorClass = "text-active"; 
                break;
            case 'pending_skip':   
                labelText = "SKIP";   
                labelColorClass = "text-skip"; 
                break;
            case 'fixed_active':   
                labelText = "FIXED";  
                labelColorClass = "text-fixed-active"; 
                break;
            case 'fixed_skip':     
                labelText = "VOID";   
                labelColorClass = "text-fixed-skip"; 
                break;
        }

        statusText.classList.add(labelColorClass);
        statusText.textContent = labelText;
        statusText.style.fontSize = "0.7em";
        statusText.style.fontWeight = "bold";
        statusText.style.marginTop = "4px";

        controlCell.appendChild(statusText);
        row.appendChild(controlCell);

        // [右カラム] 日付セル
        week.days.forEach((day, dayIndex)=> {
            const cell = document.createElement('div');
            cell.className = 'cal-cell-day'; // CSS: .cal-cell-day
            cell.textContent = day.getDate().toString();

            // 今月以外の日付は薄く
            if (day.getMonth() !== currentMonth) {
                cell.style.opacity = '0.3';
            }

            // 確定済み(Fixed)の週は背景色を変えて「ロック感」を出す
            if (state.startsWith('fixed')) {
                cell.style.backgroundColor = '#f9f9f9';
                cell.style.color = '#888';
            }

            // TODO: ここに実際のシフト内容 (Activeの場合) を描画する処理が入ります
            // if (state === 'fixed_active' || state === 'pending_active') { ... }
            // ★★★ ここが追加部分: 確定シフトの描画 ★★★
            // 「この週のデータが存在する」かつ「状態が FIXED ACTIVE」の場合のみ描画
            const weekShift = shiftData.weeks[i];

            if (state === 'fixed_active' && weekShift /* || weekShift != null  ここの条件はテスト用 */ ) {
                const dailyShift = weekShift.days[dayIndex];
                if (dailyShift) {
                    // 午前
                    if (dailyShift.morning.length > 0) {
                        const mBadge = document.createElement('div');
                        mBadge.style.fontSize = '0.75em';
                        mBadge.style.backgroundColor = '#e3f2fd'; // 薄い青
                        mBadge.style.color = '#0d47a1';
                        mBadge.style.padding = '2px 4px';
                        mBadge.style.borderRadius = '3px';
                        mBadge.style.marginBottom = '2px';
                        mBadge.textContent = `AM: ${dailyShift.morning.join(', ')}`;
                        cell.appendChild(mBadge);
                    }
                    // 午後
                    if (dailyShift.afternoon.length > 0) {
                        const aBadge = document.createElement('div');
                        aBadge.style.fontSize = '0.75em';
                        aBadge.style.backgroundColor = '#fce4ec'; // 薄いピンク
                        aBadge.style.color = '#c2185b';
                        aBadge.style.padding = '2px 4px';
                        aBadge.style.borderRadius = '3px';
                        aBadge.textContent = `PM: ${dailyShift.afternoon.join(', ')}`;
                        cell.appendChild(aBadge);
                    }
                }
            }

            row.appendChild(cell);
        });

        mount.appendChild(row);
    });
}

function calculateCalendarDates(year: number, month: number) {
    const weeks = [];
    const firstDay = new Date(year, month, 1);
    const dayOfWeek = (firstDay.getDay() + 6) % 7;
    const startDate = new Date(firstDay);
    startDate.setDate(firstDay.getDate() - dayOfWeek);
    
    const currentProcessDate = new Date(startDate);
    let weekCounter = 1;

    while (true) {
        const weekDays: Date[] = [];
        let hasCurrentMonthDay = false;
        for (let i = 0; i < 7; i++) {
            const d = new Date(currentProcessDate);
            weekDays.push(d);
            if (d.getMonth() === month) hasCurrentMonthDay = true;
            currentProcessDate.setDate(currentProcessDate.getDate() + 1);
        }
        if (!hasCurrentMonthDay && weeks.length > 0) break;
        weeks.push({ weekId: `${year}-W${weekCounter}`, days: weekDays });
        weekCounter++;
    }
    return weeks;
}

function setupEventListeners() {
    // 1. プラン選択 (Plan Select)
    const planSelect = document.getElementById('plan-select');
    if (planSelect) {
        planSelect.addEventListener('change', (e) => {
            const val = (e.target as HTMLSelectElement).value;
            if (val) handlePlanChange(parseInt(val));
        });
    }

    // 2. プラン作成 (Create Plan)
    const createPlanBtn = document.getElementById('create-plan-btn');
    if (createPlanBtn) {
        createPlanBtn.addEventListener('click', createNewPlan);
    }

    // 3. 画面切り替え (View Switching)
    document.getElementById('switch-viewer')?.addEventListener('click', () => {
        document.getElementById('view-calendar')!.style.display = 'block';
        document.getElementById('view-config')!.style.display = 'none';
    });

    document.getElementById('switch-config')?.addEventListener('click', () => {
        document.getElementById('view-calendar')!.style.display = 'none';
        document.getElementById('view-config')!.style.display = 'block';
        reloadConfig();
    });

    // document.getElementById('switch-viewer')?.addEventListener('click', () => switchView('calendar'));
    // document.getElementById('switch-config')?.addEventListener('click', () => switchView('config'));

    // ============================================================
    // ★ ここに追加: Add Group & Add Rule ボタンのフック
    // ============================================================

    // Add Group Button
    const addGroupBtn = document.getElementById('add-group-btn');
    if (addGroupBtn) {
        addGroupBtn.addEventListener('click', () => {
            console.log("Add Group Clicked"); // デバッグ用
            addNewGroup();
        });
    }

    // Add Rule Button
    const addRuleBtn = document.getElementById('add-rule-btn');
    if (addRuleBtn) {
        addRuleBtn.addEventListener('click', () => {
            console.log("Add Rule Clicked"); // デバッグ用
            addNewRule();
        });
    }

    // ============================================================

    // 4. カレンダー操作 (Calendar Actions)
    document.getElementById('prev-btn')?.addEventListener('click', () => {
        currentMonth--;
        if(currentMonth < 0) { currentMonth = 11; currentYear--; }
        renderCalendarView();
    });

    document.getElementById('next-btn')?.addEventListener('click', () => {
        currentMonth++;
        if(currentMonth > 11) { currentMonth = 0; currentYear++; }
        renderCalendarView();
    });

    // Generate Button
    // document.getElementById('generate-btn')?.addEventListener('click', handleGenerate);
    //
    document.getElementById('modal-cancel-btn')?.addEventListener('click', closeModal);
    document.getElementById('modal')?.addEventListener('click', (e) => {
        if ((e.target as HTMLElement).id === 'modal') closeModal();
    });
}

// Global Exports for onclick
(window as any).removeGroup = removeGroup;
(window as any).updateGroupName = updateGroupName;
(window as any).removeMember = removeMember;
(window as any).removeRule = removeRule;

(window as any).removeAssignment = removeAssignment;
(window as any).updateMemberName = updateMemberName;
