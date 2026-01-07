import {
    $init,
    shiftManager,
} from "./target/jco/component_features.js";

import type {
        ShiftTime,
        ShiftWeekday,
} from "./target/jco/interfaces/component-component-features-shift-manager.d";

/* ==========================================================================
   1. UTILITIES & CONSTANTS (ユーティリティと定数)
   ========================================================================== */
function el(tag: any, props = {}, ...children: any[]) {
    const element = document.createElement(tag);
    for (const [key, value] of Object.entries(props)) {
        if (key === 'className') element.className = value;
        else if (key === 'style' && typeof value === 'object') Object.assign(element.style, value);
        else if (key.startsWith('on') && typeof value === 'function') element.addEventListener(key.substring(2).toLowerCase(), value);
        else element.setAttribute(key, value);
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

    for (let i = 0; i < startOffset; i++) { tr.appendChild(el('td', { className: 'diff-month' })); count++; }

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
                        value: memo, 
                        placeholder: 'Memo', 
                        oninput: (e: Event) => { 
                                const target = e.target as HTMLInputElement;
                                updateSlotMemo(state, gIdx, sIdx, target.value) 
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
                    onclick: () => addSlot(state, gIdx) }, '+ Add Slot')
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
                                        updateRuleName(state, rIdx, target.value) }}),
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

function initApp(state: shiftManager.ShiftManager) {
    // switch Viewer <-> Config
    document.getElementById('switch-viewer')!.onclick = () => {
            switchView(state, "calendar");
    }

    document.getElementById('switch-config')!.onclick = () => {
            switchView(state, "config")
    }
    // Calendar Controls

// TODO
// TODO
/*
    document.getElementById('prev-btn')!.onclick = () => {
            state.changePrevMonth();
            renderCalendar(state);
    };
    
    document.getElementById('next-btn')!.onclick = () => {
            state.changeNextMonth()
            renderCalendar();
    };

    document.getElementById('generate-btn')!.onclick = generateSchedule;
*/

    // Config Controls
    document.getElementById('add-group-btn')!.onclick = () => addNewGroup(state);
    document.getElementById('add-rule-btn')!.onclick = () => addNewRule(state);

    // Modal Controls
    document.getElementById('modal-cancel-btn')!.onclick = closeModal;
    document.getElementById('modal')!.onclick = (e: Event) => { 
        const target = e.target as HTMLInputElement;
        if(target.id === 'modal') 
        closeModal(); 
    };

    // Initial Render
    renderConfig(state);
// TODO
// TODO
/*
    renderCalendar(state);
*/
}

$init.then(() => {
    let state = new shiftManager.ShiftManager();

    initApp(state);
    // state.addNewGroup();
    // state.addNewGroup();

    //console.log(state.getStaffGroups());

    // let submit_btn = document.getElementById("submit-btn");
    // let add_slot_btn = document.getElementById("add-slot");

    // submit_btn?.addEventListener("click", () => {
    //         console.log("submit_btn pushed");
    //         console.log(state.getStaffGroups());
    // })
    // 
    // add_slot_btn?.addEventListener("click", () => {
    //         console.log("add slot to 0")
    //         state.addSlot(0);
    // })
})

