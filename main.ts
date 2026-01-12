// main.ts
//
// ロジックのエントリーポイント
//
//

import {
    $init,
    shiftManager,
} from "./target/jco/component_features.js";

import type {
        ShiftTime,
        ShiftWeekday,
        DailyShiftOut,
        WeeklyShiftOut
} from "./target/jco/interfaces/component-component-features-shift-manager.d";

/* ==========================================================================
   1. UTILITIES & CONSTANTS (ユーティリティと定数)
   ========================================================================== */

function el(tag: any, props = {}, ...children: any[]) {
    const element = document.createElement(tag);
    for (const [key, value] of Object.entries(props)) {
        if (key === 'className')
                element.className = value;
        else if (key === 'style' && typeof value === 'object') 
                Object.assign(element.style, value);
        else if (key.startsWith('on') && typeof value === 'function') 
                element.addEventListener(key.substring(2).toLowerCase(), value);
        else 
                element.setAttribute(key, value);
    }
    children.forEach(child => {
        if (typeof child === 'string' || typeof child === 'number') element.textContent = child;
        else if (child instanceof Node) element.appendChild(child);
    });
    return element;
}

function getGroupColor(index: number) {
    const palette = ['#e67e22', '#27ae60', '#2980b9', '#8e44ad', '#c0392b', '#16a085', '#d35400', '#2c3e50'];
    return index < palette.length ? palette[index] : `hsl(${(index * 137.5) % 360}, 65%, 45%)`;
}

const getGroupPrefix = (idx: number) => String.fromCharCode(97 + idx); 

const days: ShiftWeekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];

const ShiftTimeConst: Record<string, ShiftTime> = {
    Morning: 'morning',
    Afternoon: 'afternoon',
} as const;

// Modal Context State
type ModalContext = {
    rIdx: number;
    day: ShiftWeekday;
    shift: ShiftTime;
};
let modalCtx: ModalContext | null = null;

/* ==========================================================================
   3. LOGIC & ACTION FUNCTIONS (ロジックと操作関数)
   ========================================================================== */

/* --- View Switching --- */
function switchView(state: shiftManager.ShiftManager, viewName:string) {
    document.querySelectorAll<HTMLElement>('.view-btn').forEach(btn => {
        if (btn.innerText.toLowerCase().includes(viewName)) btn.classList.add('active');
        else btn.classList.remove('active');
    });
    document.querySelectorAll('.view-section').forEach(sec => sec.classList.remove('active-view'));
    document.getElementById(`view-${viewName}`)!.classList.add('active-view');
    
    if (viewName === 'calendar') {
        updateRuleSelect(state);

        // TODO
        // TODO
        /*
        renderCalendar();
       */
    }
}

/* --- Generator Logic --- */
function updateRuleSelect(state: shiftManager.ShiftManager) {
    const select = document.getElementById('rule-select');
    if (!select) return; // エラー回避
    select.innerHTML = '';
    state.getRules().forEach((rule, idx) => {
        const option = document.createElement('option');
        option.value = idx.toString();
        option.textContent = rule.name;
        select.appendChild(option);
    });
}

function resolveStaffId(state: shiftManager.ShiftManager, idStr: String) {
    if (!idStr) return null;
    const gPrefix = idStr.charAt(0);
    const sIdx = parseInt(idStr.substring(1));
    const gIdx = gPrefix.charCodeAt(0) - 97;

    const group = state.getStaffGroups()[gIdx];
    if (!group || group.slots[sIdx] === undefined) {
        return { name: idStr + "?", groupIdx: 99 };
    }
    
    const memo = group.slots[sIdx];
    const dispName = memo ? memo : `${group.name}-${sIdx}`;
    
    return { name: dispName, groupIdx: gIdx };
}


/* --- Config CRUD Actions --- */
function addNewGroup(state: shiftManager.ShiftManager) {
    state.addNewGroup();
    renderConfig(state); 
}

function removeGroup(state: shiftManager.ShiftManager, i:number) {
    if(confirm("Shift IDs?")) {
        state.removeGroup(i)
        renderConfig(state); 
    } 
}

function updateGroupName(state: shiftManager.ShiftManager, i:number, v:string) {
    state.updateGroupName(i, v)
    renderConfig(state);
}

function addSlot(state:shiftManager.ShiftManager, i: number) {
    state.addSlot(i)
    renderConfig(state);
}

function removeSlot(state: shiftManager.ShiftManager, g:number, s:number) {
    state.removeSlot(g, s);
    renderConfig(state); 
}

function updateSlotMemo(state: shiftManager.ShiftManager, g:number, s:number, v:string) { 
    state.updateSlotMemo(g, s, v);
    renderJSON(state); 
}

function addNewRule(state: shiftManager.ShiftManager) {
    state.addRule();
    renderConfig(state); 
}

function removeRule(state: shiftManager.ShiftManager, i:number) {
    if(confirm("Del?")) { 
        state.removeRule(i);
        renderConfig(state); 
    }
}

function updateRuleName(state: shiftManager.ShiftManager, i:number, v: string) {
    state.updateRuleName(i, v);
    renderJSON(state); 
    updateRuleSelect(state); 
}

function removeAssignment(state: shiftManager.ShiftManager, r: number, d:shiftManager.ShiftWeekday, s: shiftManager.ShiftTime, i:number) { 
    state.removeRuleAssignment(r, d, s, i);
    renderConfig(state);
}

/* --- Modal Actions --- */
function openModal(state: shiftManager.ShiftManager, rIdx:number, day: ShiftWeekday, shift: ShiftTime) {
    const modalEl = document.getElementById('modal');
    const modalListEl = document.getElementById('modal-list')!;

    modalCtx = { rIdx, day, shift };
    modalListEl.replaceChildren();

    state.getStaffGroups().forEach((group, gIdx) => {
        const prefix = getGroupPrefix(gIdx);
        const color = getGroupColor(gIdx);
        const container = el('div', { style: { marginBottom: "20px" } }, 
            el('div', { style: { color: color, fontWeight: "bold", marginBottom: "5px" } }, `${group.name} (${prefix})`)
        );
        const grid = el('div', { className: 'selection-grid' });

        group.slots.forEach((memo, sIdx) => {
            const idStr = `${prefix}${sIdx}`;
            // モーダルではメモがある場合は (メモ) を付記
            const label = memo ? `${idStr}` : idStr;
            grid.appendChild(el('div', { 
                className: 'selection-btn', 
                style: { borderLeftColor: color }, 
                onclick: () => confirmAssignment(state, gIdx, sIdx) 
            }, label));
        });
        
        container.appendChild(grid);
        modalListEl.appendChild(container);
    });
    
    modalEl!.style.display = 'flex';
}

function confirmAssignment(state: shiftManager.ShiftManager, staffGroupId: number, shiftStaffIndex:number){
    state.addRuleAssignment(
        modalCtx!.rIdx,
        modalCtx!.day,
        modalCtx!.shift,
        staffGroupId, 
        shiftStaffIndex
    );
    document.getElementById('modal')!.style.display = 'none';
    renderConfig(state);
}

function closeModal() {
    document.getElementById('modal')!.style.display = 'none';
}

/* ==========================================================================
   4. RENDER FUNCTIONS (描画関数)
   ========================================================================== */

// TODO
// TODO
/*
function renderCalendar(state: shiftManager.ShiftManager) {
    const mount = document.getElementById('calendar-mount')!;
    const label = document.getElementById('current-month-label')!;
    const year = state.getYear();
    const month = state.getMonth();

    label.textContent = new Date(year, month, 1).toLocaleDateString('en-US', { year: 'numeric', month: 'long' });

    const firstDay = new Date(year, month, 1).getDay();
    const startOffset = (firstDay === 0 ? 6 : firstDay - 1);
    const totalDays = new Date(year, month + 1, 0).getDate();
    
    const thead = el('thead', {}, el('tr', {},
        el('th', {}, 'MON'), el('th', {}, 'TUE'), el('th', {}, 'WED'),
        el('th', {}, 'THU'), el('th', {}, 'FRI'), el('th', {style:{color:'#e67e22'}}, 'SAT'),
        el('th', {style:{color:'#e74c3c'}}, 'SUN')
    ));

    const tbody = el('tbody');
    let tr = el('tr');
    let count = 0;

    for (let i = 0; i < startOffset; i++) { 
            tr.appendChild(el('td', { className: 'diff-month' })); 
            count++; 
    }

    const today = new Date();
    for (let d = 1; d <= totalDays; d++) {
        if (count % 7 === 0 && count !== 0) { tbody.appendChild(tr); tr = el('tr'); }
        
        const dateStr = `${year}-${String(month + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
        const dayData = state.scheduleData[dateStr] || { m: [], a: [] };

        // state.getRuleAssignment(ruleIdx, day, shiftTime)

        const cellContent = [];
        cellContent.push(el('span', { className: 'date-label' }, d));

        [ShiftTimeConst.Morning , ShiftTimeConst.Afternoon].forEach(shiftType => {
            let holl_list = state.getRuleAssignment(, day, shiftType);
            if (holl_list && holl_list.length > 0) {
                const chips = holl_list.map(s => el('span', { 
                    className: 'staff-chip', 
                    style:{borderLeftColor:getGroupColor(s.staffGroupId)}, title: s.name 
                }, s.name));
                cellContent.push(el('div', { className: 'shift-section' }, 
                    el('div', { className: 'shift-label' }, shiftType === ShiftTimeConst.Morning ? 'AM' : 'PM'), ...chips
                ));
            }
        });

        const td = el('td', {}, ...cellContent);
        if (today.getFullYear()===year && today.getMonth()===month && today.getDate()===d) td.classList.add('today');
        tr.appendChild(td);
        count++;
    }
    while (count % 7 !== 0) { tr.appendChild(el('td', { className: 'diff-month' })); count++; }
    tbody.appendChild(tr);

    mount.replaceChildren(el('table', { className: 'calendar-table' }, thead, tbody));
}
*/

// --- Imports (Wasm generated files) ---
// import { generateMonthlyView } from "./shift_engine.js"; 
// ※ 実際はjco等で生成されたファイルをimportします

// --- State Management ---


// --- Calendar Logic ---
interface AppState {
    year: number;
    month: number; // 1-12
    weekSkipState: Record<string, boolean>; // key: "YYYY-Wxx", value: isActive
    baseDelta: number;
}

// --- Global State for UI Control ---
// Wasmに反映する前のチェックボックスの状態を一時保持するリスト
let pendingSkipFlags: boolean[] = [];

/**
 * カレンダー描画関数
 * Wasmから現在のシフト状態を取得し、pendingSkipFlags と合わせて描画します
 */
function renderCalendar(manager: shiftManager.ShiftManager) {
    const mount = document.getElementById('calendar-mount');
    const label = document.getElementById('current-month-label');

    // ラベル更新
    if (label) {
        label.textContent = new Date(manager.getYear(), manager.getMonth(), 1)
            .toLocaleDateString('en-US', { year: 'numeric', month: 'long' });
    }
    
    if (!mount) return;
    mount.innerHTML = '';

    // 1. カレンダーの日付計算 (月曜始まり)
    const weeksData = calculateCalendarDates(manager.getYear(), manager.getMonth());

    // 2. フラグ配列の初期化 (月が変わった場合などの整合性確保)
    if (pendingSkipFlags.length !== weeksData.length) {
        // デフォルトは全て Active (true) にする、あるいはWasmの前回状態から復元も可
        pendingSkipFlags = new Array(weeksData.length).fill(true);
    }

    // 3. Wasmからシフトデータを取得 (Generate済みの場合データが入る)
    // 型: (WeeklyShiftOut | undefined)[] 
    const shiftList = manager.getMonthlyShift(); 

    // 4. DOM生成
    const fragment = document.createDocumentFragment();

    weeksData.forEach((week, index) => {
        // UI上のチェック状態
        const isUiActive = pendingSkipFlags[index];
        
        // Wasm上のデータ (Generateされていれば存在する)
        // Wasm側で option<weekly-shift-out> なので、TS側では undefined チェックが必要
        const weekShiftData = shiftList[index]; 
        console.log("weekShiftData", weekShiftData)

        // 行コンテナ
        // UIでスキップ選択中、またはWasmデータが無い(None)場合は skipped スタイル
        const row = document.createElement('div');
        const visualActive = isUiActive && (weekShiftData !== undefined);
        row.className = `cal-week-row ${visualActive ? 'active' : 'skipped'}`;

        // --- [左列] コントロール ---
        const controlCell = document.createElement('div');
        controlCell.className = 'cal-cell-control';

        // Checkbox
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = isUiActive;
        checkbox.addEventListener('change', (e) => {
            const checked = (e.target as HTMLInputElement).checked;
            // TS側の一時フラグを更新
            pendingSkipFlags[index] = checked;
            // 即座に再描画 (グレーアウト処理のため。WasmへのApplyはまだしない)
            renderCalendar(manager); 
        });

        // Shift Label (データがあれば表示)
        const labelDiv = document.createElement('div');
        labelDiv.className = `shift-badge ${visualActive ? 'active' : 'skipped'}`;
        // 簡易表示: データがあるかどうかで判定
        labelDiv.textContent = visualActive ? "Active" : "Skip";

        controlCell.appendChild(checkbox);
        controlCell.appendChild(labelDiv);
        row.appendChild(controlCell);

        // --- [右7列] 日付セル ---
        week.days.forEach((day, dayIndex) => {
            const dayCell = document.createElement('div');
            dayCell.className = 'cal-cell-day';

            // 日付数値
            const numSpan = document.createElement('span');
            numSpan.className = 'date-num';
            numSpan.textContent = day.getDate().toString();
            if (day.getMonth() !== manager.getMonth()) {
                dayCell.style.opacity = '0.4';
            }
            dayCell.appendChild(numSpan);

            // シフト割り当ての描画
            // visualActive かつ シフトデータが存在する場合のみ描画
            if (visualActive && weekShiftData) {
                // weekly-shift-out の mon, tue... を index (0-6) で引けるようにする
                const dailyShift = getDailyShiftByIndex(weekShiftData, dayIndex);
                if (dailyShift) {
                    renderDailyShift(dayCell, dailyShift);
                }
            }

            row.appendChild(dayCell);
        });

        fragment.appendChild(row);
    });

    mount.appendChild(fragment);
}

/**
 * WeeklyShiftOut (struct) から index (0=Mon, ... 6=Sun) で DailyShiftOut を取り出すヘルパー
 */
function getDailyShiftByIndex(week: WeeklyShiftOut, index: number): DailyShiftOut | null {
    switch (index) {
        case 0: return week.mon;
        case 1: return week.tue;
        case 2: return week.wed;
        case 3: return week.thu;
        case 4: return week.fri;
        case 5: return week.sat;
        case 6: return week.sun;
        default: return null;
    }
}

/**
 * 1日分のシフト(午前・午後)をセルに描画するヘルパー
 */
function renderDailyShift(container: HTMLElement, daily: DailyShiftOut) {
    // 午前 (M)
    if (daily.m.length > 0) {
        const mBadge = document.createElement('div');
        mBadge.className = 'shift-slot slot-morning';
        mBadge.textContent = `AM: ${daily.m.map(s => s.name).join(',')}`;
        container.appendChild(mBadge);
    }
    // 午後 (A)
    if (daily.a.length > 0) {
        const aBadge = document.createElement('div');
        aBadge.className = 'shift-slot slot-afternoon';
        aBadge.textContent = `PM: ${daily.a.map(s => s.name).join(',')}`;
        container.appendChild(aBadge);
    }
}



// // --- Logic: 月曜始まりのカレンダー計算 ---
function calculateCalendarDates(year: number, month: number) {
    // month: 0 = January, 11 = December
    const weeks = [];

    // 月の初日 (JavaScriptのDateも月は0始まりなのでそのまま渡す)
    const firstDay = new Date(year, month, 1);

    // カレンダーの開始日を決定（その月の1日を含む週の月曜日まで戻る）
    // firstDay.getDay(): 0(Sun) ... 6(Sat)
    // 月曜始まり(Mon=0)にするための計算: (day + 6) % 7
    const dayOfWeek = (firstDay.getDay() + 6) % 7;

    const startDate = new Date(firstDay);
    // 日付を戻す (setDateは自動的に前月へ繰り越してくれる)
    startDate.setDate(firstDay.getDate() - dayOfWeek);

    const currentProcessDate = new Date(startDate);

    // 週番号のカウンタ
    let weekCounter = 1;

    // 無限ループで回し、その週が「完全に翌月以降」になったら抜ける
    while (true) {
        const weekDays: Date[] = [];
        let hasCurrentMonthDay = false;

        // 1週間(7日)分の日付を取得
        for (let i = 0; i < 7; i++) {
            // 日付オブジェクトを複製してリストに追加
            const d = new Date(currentProcessDate);
            weekDays.push(d);

            // その日が「指定された月」に含まれるかチェック
            // month引数(0-11) と d.getMonth()(0-11) を直接比較
            if (d.getMonth() === month) {
                hasCurrentMonthDay = true;
            }

            // 次の日へ進める
            currentProcessDate.setDate(currentProcessDate.getDate() + 1);
        }

        // その週の中に、今月(指定されたmonth)の日が1日もなければ終了
        // (＝カレンダーの末尾を超えた)
        if (!hasCurrentMonthDay && weeks.length > 0) {
            break;
        }

        weeks.push({
            weekId: `${year}-W${weekCounter}`, // 簡易ID
            days: weekDays
        });
        weekCounter++;
    }

    return weeks;
}

// calendar ==========================

function renderConfig(state: shiftManager.ShiftManager) { 
    renderGroups(state); 
    renderRules(state); 
    renderJSON(state); 
    updateRuleSelect(state); 
}

function renderGroups(state: shiftManager.ShiftManager) {
    const container = document.getElementById('staff-groups-container')!;
    container.replaceChildren();
    state.getStaffGroups().forEach((group, gIdx) => {
        const prefix = getGroupPrefix(gIdx);
        const color = getGroupColor(gIdx);
        const slotListContainer = el('div', { className: 'slot-list' });
        
        group.slots.forEach((memo, sIdx) => {
            slotListContainer.appendChild(el('div', { className: 'slot-item' },
                el('span', { className: 'slot-idx' }, `${sIdx}:`),
                el('input', { 
                        type: 'text', 
                        className: 'slot-input', 
                        value: memo.name, 
                        placeholder: 'Memo', 
                        oninput: (e: Event) => { 
                                const target = e.target as HTMLInputElement;

                                updateSlotMemo(state, gIdx, sIdx, target.value);
                                // target.textContent = ;
                        }
                }),
                el('button', { 
                        className: 'btn btn-danger btn-sm', 
                        onclick: () => removeSlot(state, gIdx, sIdx) }, '×'
                  )
            ));
        });
        
        container.appendChild(el('div', { className: 'group-card', style: { borderTopColor: color } },
            el('div', { className: 'group-header' },
                el('span', { className: 'group-id-badge', style: { backgroundColor: color } }, `ID: ${prefix}`),
                el('button', { className: 'btn btn-danger btn-sm', onclick: () => removeGroup(state, gIdx) }, 'Delete')
            ),
            el('input', { 
                    type: 'text',
                    className: 'group-name-input',
                    value: group.name,
                    placeholder: 'Group Name', 
                    oninput: (e:Event) => {
                            const target = e.target as HTMLInputElement;
                            updateGroupName(state, gIdx, target.value) 
                    }
            }),
            slotListContainer,
            el('button', {
                    className: 'btn btn-outline',
                    style: { width: '100%', fontSize: '0.8em' },
                    onclick: () => { 
                            addSlot(state, gIdx);
                    } }, '+ Add Slot')
        ));
    });
}

function renderRules(state: shiftManager.ShiftManager) {
    const container = document.getElementById('rules-container')!;
    container.replaceChildren();

    state.getRules().forEach((rule, rIdx) => {
        const theadTr = el('tr', {}, el('th', { className: 'config-row-header' }, 'Shift'));
        days.forEach((d) => theadTr.appendChild(el('th', {}, d.toUpperCase())));
        
        const tbody = el('tbody');
        [ShiftTimeConst.Morning , ShiftTimeConst.Afternoon].forEach(shiftType => {
            const tr = el('tr', {});
            tr.appendChild(el('td', { className: 'config-row-header' }, shiftType === ShiftTimeConst.Morning ? 'Morning' : 'Afternoon'));

            days.forEach(day => {
                const cell = el('td', {});
                state
                .getRuleAssignment(rIdx, day, shiftType)!
                .forEach((holl, arrIdx) => {
                    const gPrefix = holl.staffGroupId;
                    const color = getGroupColor(gPrefix);

                    // Config画面では ID (a0, b1) を表示
                    const label = `${holl.staffGroupId.toString()}-${holl.shiftStaffIndex.toString()}`; 
                    
                    cell.appendChild(el('span', { 
                        className: 'chip', 
                        style: { backgroundColor: color }, 
                        title: holl, 
                        onclick: () => removeAssignment(state, rIdx, day, shiftType, arrIdx) 
                    }, label));
                });
                cell.appendChild(el('button', { className: 'add-btn-mini', onclick: () => openModal(state , rIdx, day, shiftType) }, '+'));
                tr.appendChild(cell);
            });
            tbody.appendChild(tr);
        });
        
        container.appendChild(el('div', { className: 'rule-card' },
            el('div', { className: 'rule-header' },
                el(
                        'input', 
                        { 
                                type: 'text',
                                style: { fontSize: '1.1em', fontWeight: 'bold' }, 
                                value: rule.name, 
                                oninput: (e: Event) => {
                                        const target = e.target as HTMLInputElement;
                                        updateRuleName(state, rIdx, target.value) 
                                }
                        }),
                el('button', { className: 'btn btn-danger', onclick: () => removeRule(state, rIdx) }, 'Delete Rule')
            ),
            el('table', { className: 'config-table' }, el('thead', {}, theadTr), tbody)
        ));
    });
}

function renderJSON(state: shiftManager.ShiftManager) { 
    document.getElementById('json-output')!.textContent = JSON.stringify({staffGroups: state.getStaffGroups(), rules: state.getRules()}, null, 2); 
}

/* ==========================================================================
   5. INITIALIZATION & EVENT LISTENERS (初期化とイベント設定)
   ========================================================================== */

function initApp(manager: shiftManager.ShiftManager) {
    // switch Viewer <-> Config
    document.getElementById('switch-viewer')!.onclick = () => {
            switchView(manager, "calendar");
    }

    document.getElementById('switch-config')!.onclick = () => {
            switchView(manager, "config")
    }

    // Calendar Controls
    document.getElementById('prev-btn')!.onclick = () => {
        manager.changePrevMonth();
        // 月が変わったらフラグもリセット
        pendingSkipFlags = [];
        renderCalendar(manager);
    };

    document.getElementById('next-btn')!.onclick = () => {
        manager.changeNextMonth();
        pendingSkipFlags = [];
        renderCalendar(manager);
    };

    // ★ Generate Button Implementation
    document.getElementById('generate-btn')!.onclick = () => {
        console.log("Applying Rules:", pendingSkipFlags);

        try {
            // 1. UIで設定されたフラグリスト(pendingSkipFlags)をWasmに渡す
            //    WIT定義: apply-month-shift: func(skip-flags: list<bool>)
            manager.applyMonthShift(pendingSkipFlags);

            // 2. 適用後の状態を再描画 (getMonthlyShiftの結果が変わるはず)
            renderCalendar(manager);
        } catch (e) {
            console.error("Failed to generate shift:", e);
            alert("シフト生成に失敗しました");
        }
    };

    // Config / Modal Controls (省略)

    // Config Controls
    document.getElementById('add-group-btn')!.onclick = () => addNewGroup(manager);
    document.getElementById('add-rule-btn')!.onclick = () => addNewRule(manager);

    // Modal Controls
    document.getElementById('modal-cancel-btn')!.onclick = closeModal;
    document.getElementById('modal')!.onclick = (e: Event) => { 
        const target = e.target as HTMLInputElement;
        if(target.id === 'modal') 
        closeModal(); 
    };

    // Initial Render
    renderCalendar(manager);
}


$init.then(() => {
    let state = new shiftManager.ShiftManager();

    initApp(state);
})

