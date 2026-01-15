// main.ts
//
// ロジックのエントリーポイント
//
//

import {
    $init,
    shiftManager,
    asyncExampleFunc
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
function switchView(manager: shiftManager.ShiftManager, viewName:string) {
    document.querySelectorAll<HTMLElement>('.view-btn').forEach(btn => {
        if (btn.innerText.toLowerCase().includes(viewName)) btn.classList.add('active');
        else btn.classList.remove('active');
    });
    document.querySelectorAll('.view-section').forEach(sec => sec.classList.remove('active-view'));
    document.getElementById(`view-${viewName}`)!.classList.add('active-view');
    
    if (viewName === 'calendar') {
        updateRuleSelect(manager);
    }
}

/* --- Generator Logic --- */
function updateRuleSelect(manager: shiftManager.ShiftManager) {
    const select = document.getElementById('rule-select');
    if (!select) return; // エラー回避
    select.innerHTML = '';
    manager.getRules().forEach((rule, idx) => {
        const option = document.createElement('option');
        option.value = idx.toString();
        option.textContent = rule.name;
        select.appendChild(option);
    });
}

/* --- Config CRUD Actions --- */
function addNewGroup(manager: shiftManager.ShiftManager) {
    manager.addNewGroup();
    renderConfig(manager); 
}

function removeGroup(manager: shiftManager.ShiftManager, i:number) {
    if(confirm("Shift IDs?")) {
        manager.removeGroup(i)
        renderConfig(manager); 
    } 
}

function updateGroupName(manager: shiftManager.ShiftManager, i:number, v:string) {
    manager.updateGroupName(i, v)
    renderConfig(manager);
}

function addSlot(manager:shiftManager.ShiftManager, i: number) {
    manager.addSlot(i)
    renderConfig(manager);
}

function removeSlot(manager: shiftManager.ShiftManager, g:number, s:number) {
    manager.removeSlot(g, s);
    renderConfig(manager); 
}

function updateSlotMemo(manager: shiftManager.ShiftManager, g:number, s:number, v:string) { 
    manager.updateSlotMemo(g, s, v);
    renderJSON(manager); 
}

function addNewRule(manager: shiftManager.ShiftManager) {
    manager.addWeek();
    renderConfig(manager); 
}

function removeRule(manager: shiftManager.ShiftManager, i:number) {
    if(confirm("Del?")) { 
        manager.removeRule(i);
        renderConfig(manager); 
    }
}

function updateRuleName(manager: shiftManager.ShiftManager, i:number, v: string) {
    manager.updateRuleName(i, v);
    renderJSON(manager); 
    updateRuleSelect(manager); 
}

function removeAssignment(manager: shiftManager.ShiftManager, r: number, d:shiftManager.ShiftWeekday, s: shiftManager.ShiftTime, i:number) { 
    manager.removeRuleAssignment(r, d, s, i);
    renderConfig(manager);
}

/* --- Modal Actions --- */
function openModal(manager: shiftManager.ShiftManager, rIdx:number, day: ShiftWeekday, shift: ShiftTime) {
    const modalEl = document.getElementById('modal');
    const modalListEl = document.getElementById('modal-list')!;

    modalCtx = { rIdx, day, shift };
    modalListEl.replaceChildren();

    manager.getStaffGroups().forEach((group, gIdx) => {
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
                onclick: () => confirmAssignment(manager, gIdx, sIdx) 
            }, label));
        });
        
        container.appendChild(grid);
        modalListEl.appendChild(container);
    });
    
    modalEl!.style.display = 'flex';
}

function confirmAssignment(manager: shiftManager.ShiftManager, staffGroupId: number, shiftStaffIndex:number){
    manager.addRuleAssignment(
        modalCtx!.rIdx,
        modalCtx!.day,
        modalCtx!.shift,
        staffGroupId, 
        shiftStaffIndex
    );
    document.getElementById('modal')!.style.display = 'none';
    renderConfig(manager);
}

function closeModal() {
    document.getElementById('modal')!.style.display = 'none';
}

/* ==========================================================================
   4. RENDER FUNCTIONS (描画関数)
   ========================================================================== */

// --- Global State for UI Control ---
// Wasmに反映する前のチェックボックスの状態を一時保持するリスト

/**
 * カレンダー描画関数
 * Wasmから現在のシフト状態を取得し、pendingSkipFlags と合わせて描画します
 */
// --- Global State ---
// API仕様に合わせる: True = Skip (休み), False = Active (稼働)

// --- Global State ---
type skip_states = "fixed_skipped" | "fixed_active" | "pending_active"| "pending_skipped";
let pendingSkipFlags2: skip_states[] = [];

function renderCalendar(manager: shiftManager.ShiftManager) {
    // 1. ラベル更新
    const label = document.getElementById('current-month-label');
    if (label) {
        const date = new Date(manager.getYear(), manager.getMonth(), 1);
        label.textContent = date.toLocaleDateString('ja-JP', { year: 'numeric', month: 'long' });
    }

    const mount = document.getElementById('calendar-mount');
    if (!mount) return;
    mount.innerHTML = '';

    const weeksData = calculateCalendarDates(manager.getYear(), manager.getMonth());
    
    // 2. Managerから最新情報を取得
    const savedSkipFlags = manager.getSkipFlags(); 
    console.log("savedSkipFlags", savedSkipFlags);
    const shiftList = manager.getMonthlyShift(); 

    // --- 配列初期化 (同期) ロジック ---
    if (pendingSkipFlags2.length !== weeksData.length) {
        pendingSkipFlags2 = [];

        // ★修正: 配列全体の長さ判定(isCurrentConfigSaved)を廃止し、
        // 週ごとのインデックスで判定するように変更

        weeksData.forEach((week, i) => {
            // A. この週に対する保存設定が「存在するか」確認
            const savedFlag = savedSkipFlags[i];

            // B. シフトデータが存在するか確認
            const hasShiftData = shiftList[i] !== undefined;

            if (savedFlag !== undefined) {
                // ■ ケース1: 保存された設定がある場合 (最優先)
                // savedSkipFlags[i] が true ならスキップ、false なら稼働
                if (savedFlag) {
                    pendingSkipFlags2.push('fixed_skipped');
                } else {
                    pendingSkipFlags2.push('fixed_active');
                }
            } else {
                // ■ ケース2: 保存された設定がない場合 (未生成の週)
                if (hasShiftData) {
                    // 設定はないがデータがある (前月生成分の溢れなど) -> FIXED ACTIVE
                    pendingSkipFlags2.push('fixed_active');
                } else {
                    // 設定もデータもない -> これから編集する週 (READY)
                    // ※月またぎの判定(isOverlap)をここで厳密にやると「最初の月が編集できない」問題が出るため、
                    //   savedFlagがない場合は一律「編集可能」にします。
                    //   (前月でスキップされたなら、通常はsavedFlagに[true]が入ってくるはずなのでこれで動きます)
                    pendingSkipFlags2.push('pending_active');
                }
            }
        });
    }

    const fragment = document.createDocumentFragment();

    weeksData.forEach((week, index) => {
        const weekShiftData = shiftList[index];
        const skipState = pendingSkipFlags2[index];

        // --- ステータス決定 ---
        let rowClass = "";
        let statusLabel = "";
        let labelColorClass = "";
        
        // フラグによるスタイル分岐
        if (skipState == 'fixed_skipped') {
            rowClass = "status-skipped-fixed";
            statusLabel = "FIXED SKIP";
            labelColorClass = "text-fixed-skipped";
        } else if (skipState == 'pending_skipped') {
             rowClass = "status-skipped";
             statusLabel = "SKIP";
             labelColorClass = "text-pending-skipped";
        } else if (skipState == 'fixed_active'){
            rowClass = "status-decided";
            statusLabel = "FIXED";
            labelColorClass = "text-fixed-active";
        } else {
            rowClass = "status-active";
            statusLabel = "READY";
            labelColorClass = "text-pending-active";
        }

        const row = document.createElement('div');
        row.className = `cal-week-row ${rowClass}`;

        // --- [左列] コントロール ---
        const controlCell = document.createElement('div');
        controlCell.className = 'cal-cell-control';

        const switchLabel = document.createElement('label');
        switchLabel.className = 'switch';
        
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';

        // チェック状態: SkipならTrue
        if (skipState == 'fixed_skipped' || skipState == 'pending_skipped') {
            checkbox.checked = true;
        } else {
            checkbox.checked = false;
        }

        // ★要件対応: FIXEDな状態のものは変更不可にする
        if (skipState === 'fixed_skipped' || skipState === 'fixed_active') {
            checkbox.disabled = true;
            switchLabel.style.opacity = '0.6'; // 視覚的にも無効感を出す
            switchLabel.title = "確定済みのシフトです。変更するにはリセットしてください。";
        }

        checkbox.addEventListener('change', (e) => {
            const isChecked = (e.target as HTMLInputElement).checked;
            if (isChecked) {
                pendingSkipFlags2[index] = 'pending_skipped';
            } else {
                pendingSkipFlags2[index] = 'pending_active';
            }
            renderCalendar(manager); 
        });

        const slider = document.createElement('span');
        slider.className = 'slider';

        switchLabel.appendChild(checkbox);
        switchLabel.appendChild(slider);
        controlCell.appendChild(switchLabel);

        const statusText = document.createElement('span');
        statusText.className = `status-text ${labelColorClass}`;
        statusText.style.fontSize = "10px";
        statusText.style.marginTop = "4px";
        statusText.style.fontWeight = "bold";
        statusText.textContent = statusLabel;
        
        controlCell.appendChild(statusText);
        row.appendChild(controlCell);

        // --- [右7列] 日付セル ---
        week.days.forEach((day, dayIndex) => {
            const dayCell = document.createElement('div');
            dayCell.className = 'cal-cell-day';
            
            const numSpan = document.createElement('span');
            numSpan.className = 'date-num';
            numSpan.textContent = day.getDate().toString();
            if (day.getMonth() !== manager.getMonth()) dayCell.style.opacity = '0.3';
            dayCell.appendChild(numSpan);

            // シフトデータがあれば表示
            if (weekShiftData) {
                const dailyShift = getDailyShiftByIndex(weekShiftData, dayIndex);
                if (dailyShift) renderDailyShift(dayCell, dailyShift);
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

function renderConfig(manager: shiftManager.ShiftManager) { 
    renderGroups(manager); 
    renderRules(manager); 
    renderJSON(manager); 
    updateRuleSelect(manager); 
}

function renderGroups(manager: shiftManager.ShiftManager) {
    const container = document.getElementById('staff-groups-container')!;
    container.replaceChildren();
    manager.getStaffGroups().forEach((group, gIdx) => {
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

                                updateSlotMemo(manager, gIdx, sIdx, target.value);
                                // target.textContent = ;
                        }
                }),
                el('button', { 
                        className: 'btn btn-danger btn-sm', 
                        onclick: () => removeSlot(manager, gIdx, sIdx) }, '×'
                  )
            ));
        });
        
        container.appendChild(el('div', { className: 'group-card', style: { borderTopColor: color } },
            el('div', { className: 'group-header' },
                el('span', { className: 'group-id-badge', style: { backgroundColor: color } }, `ID: ${prefix}`),
                el('button', { className: 'btn btn-danger btn-sm', onclick: () => removeGroup(manager, gIdx) }, 'Delete')
            ),
            el('input', { 
                    type: 'text',
                    className: 'group-name-input',
                    value: group.name,
                    placeholder: 'Group Name', 
                    oninput: (e:Event) => {
                            const target = e.target as HTMLInputElement;
                            updateGroupName(manager, gIdx, target.value) 
                    }
            }),
            slotListContainer,
            el('button', {
                    className: 'btn btn-outline',
                    style: { width: '100%', fontSize: '0.8em' },
                    onclick: () => { 
                            addSlot(manager, gIdx);
                    } }, '+ Add Slot')
        ));
    });
}

function renderRules(manager: shiftManager.ShiftManager) {
    const container = document.getElementById('rules-container')!;
    container.replaceChildren();

    manager.getRules().forEach((rule, rIdx) => {
        const theadTr = el('tr', {}, el('th', { className: 'config-row-header' }, 'Shift'));
        days.forEach((d) => theadTr.appendChild(el('th', {}, d.toUpperCase())));
        
        const tbody = el('tbody');
        [ShiftTimeConst.Morning , ShiftTimeConst.Afternoon].forEach(shiftType => {
            const tr = el('tr', {});
            tr.appendChild(el('td', { className: 'config-row-header' }, shiftType === ShiftTimeConst.Morning ? 'Morning' : 'Afternoon'));

            days.forEach(day => {
                const cell = el('td', {});
                manager
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
                        onclick: () => removeAssignment(manager, rIdx, day, shiftType, arrIdx) 
                    }, label));
                });
                cell.appendChild(el('button', { className: 'add-btn-mini', onclick: () => openModal(manager , rIdx, day, shiftType) }, '+'));
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
                                        updateRuleName(manager, rIdx, target.value) 
                                }
                        }),
                el('button', { className: 'btn btn-danger', onclick: () => removeRule(manager, rIdx) }, 'Delete Rule')
            ),
            el('table', { className: 'config-table' }, el('thead', {}, theadTr), tbody)
        ));
    });
}

function renderJSON(manager: shiftManager.ShiftManager) { 
    document.getElementById('json-output')!.textContent = JSON.stringify({staffGroups: manager.getStaffGroups(), rules: manager.getRules()}, null, 2); 
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
        pendingSkipFlags2 = [];
        renderCalendar(manager);
    };

    document.getElementById('next-btn')!.onclick = () => {
        manager.changeNextMonth();
        pendingSkipFlags2 = [];
        renderCalendar(manager);
    };

    // ★ Generate Button Implementation
    document.getElementById('generate-btn')!.onclick = () => {
        console.log("Applying Rules:", pendingSkipFlags2);

        try {
            // 1. UIで設定されたフラグリスト(pendingSkipFlags)をWasmに渡す
            //    WIT定義: apply-month-shift: func(skip-flags: list<bool>)
            manager.applyMonthShift(pendingSkipFlags2
                .map((i) => i == 'fixed_skipped' || i == 'pending_skipped'));
        } catch (e) {
            console.error("Failed to generate shift:", e);
            alert("シフト生成に失敗しました");
        }
        // 2. 適用後の状態を再描画 (getMonthlyShiftの結果が変わるはず)
        pendingSkipFlags2 = [];
        renderCalendar(manager);
    };

    // ★新規追加: Reset Button Implementation
    document.getElementById('reset-btn')!.onclick = () => {
        const year = manager.getYear();
        const month = manager.getMonth(); // 0-11
        
        if (!confirm(`後続するシフトは今月${year}年${month + 1}月の予定に依存するため、以降の月の予定はすべてリセットされます。この操作を続けますか？`)) {
            return;
        }

        try {
            // Wasm側に実装した truncate (指定月以降削除) を呼び出す
            // ※ truncateFrom の引数がどう定義されているかに合わせてください
            // ここでは絶対週番号などを意識せず、現在の year/month 以降を消すAPIがあると仮定
            
            // もしWasm側に `truncate_from_current_month()` のようなAPIがある場合:
            // manager.truncateScheduleFromCurrentMonth();
            
            // あるいは Rust側で定義した `truncate_from(abs_week)` を呼ぶためのラッパーが必要です。
            // 簡易的に「この月の設定を空で上書きする」だけでは不十分（未来も消す必要があるため）。
            // ここでは `resetFrom` というAPIをWasmに追加したと仮定します。
            manager.resetFromThisMonth();

            // 成功したらローカル状態をクリアして再描画
            // -> savedSkipFlags が空になるため、自動的に READY (青) に戻る
            pendingSkipFlags2 = [];
            renderCalendar(manager);
            
        } catch (e) {
            console.error("Reset failed:", e);
            alert("リセットに失敗しました");
        }
    }

// --- JSON Load/Save Controls ---

    const fileInput = document.getElementById('config-file-input') as HTMLInputElement;
    const importBtn = document.getElementById('import-json-btn');
    const downloadBtn = document.getElementById('download-json-btn');

    // 1. Loadボタンが押されたら、隠しinputをクリックさせる
    if (importBtn) {
        importBtn.onclick = () => {
            fileInput.click();
        };
    }

    // 2. ファイルが選択された時の処理
    if (fileInput) {
        fileInput.onchange = async (e) => {
            const target = e.target as HTMLInputElement;
            const file = target.files?.[0];

            if (!file) return;

            try {
                // ファイルの中身をテキストとして読み取る
                const jsonText = await file.text();

                // WASMのメソッドを呼び出してロード
                // WIT定義: load-config-from-json: func(json-setting: string) -> result<_, string>;
                // jcoのバインディングでは、ResultのErrは例外としてスローされます
                manager.loadConfigFromJson(jsonText);

                // 成功したらUIを更新
                renderConfig(manager);
                alert(`設定ファイルを読み込みました: ${file.name}`);

            } catch (err: any) {
                console.error("Config Load Error:", err);
                // Rust側から返されたエラーメッセージを表示
                alert(`設定の読み込みに失敗しました:\n${err}`);
            } finally {
                // 同じファイルを再度選択できるように値をリセット
                fileInput.value = '';
            }
        };
    }

    // 3. Save (Download) ボタンの処理
    if (downloadBtn) {
        downloadBtn.onclick = () => {
            // 現在の設定を取得
            const dataStr = JSON.stringify({
                staffGroups: manager.getStaffGroups(),
                rules: manager.getRules()
            }, null, 2);

            // Blobを作成してダウンロードリンクを生成
            const blob = new Blob([dataStr], { type: "application/json" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `shift_config_${new Date().toISOString().slice(0, 10)}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        };
    }

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
    let manager = new shiftManager.ShiftManager();

    initApp(manager);

    document.getElementById('test-button')!.onclick = (e) => {
            let a = manager.outputCalendarManagerData();
            console.log("get time line", a);
    };
})

