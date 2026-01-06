/* ==========================================================================
   1. UTILITIES & CONSTANTS (ユーティリティと定数)
   ========================================================================== */
function el(tag, props = {}, ...children) {
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

function getGroupColor(index) {
    const palette = ['#e67e22', '#27ae60', '#2980b9', '#8e44ad', '#c0392b', '#16a085', '#d35400', '#2c3e50'];
    return index < palette.length ? palette[index] : `hsl(${(index * 137.5) % 360}, 65%, 45%)`;
}

const getGroupPrefix = (idx) => String.fromCharCode(97 + idx); 
const days = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];


/* ==========================================================================
   2. STATE MANAGEMENT (状態管理)
   ========================================================================== */
const state = {
    staffGroups: [
        { name: "Kitchen", slots: ["Leader", "Sub", ""] },
        { name: "Hall", slots: ["Manager", "PartTime", ""] }
    ],
    rules: [
        {
            name: "Standard Week",
            schedule: {
                mon: { m: ['a0', 'b0'], a: ['b1'] },
                tue: { m: [], a: ['a1'] },
                wed: { m: [], a: [] },
                thu: { m: ['a1'], a: [] },
                fri: { m: ['b1'], a: ['a0'] },
                sat: { m: [], a: [] },
                sun: { m: [], a: [] }
            }
        }
    ],
    year: 2026,
    month: 0,
    scheduleData: {} 
};

// Modal Context State
let modalCtx = null;


/* ==========================================================================
   3. LOGIC & ACTION FUNCTIONS (ロジックと操作関数)
   ========================================================================== */

/* --- View Switching --- */
function switchView(viewName) {
    document.querySelectorAll('.view-btn').forEach(btn => {
        if (btn.innerText.toLowerCase().includes(viewName)) btn.classList.add('active');
        else btn.classList.remove('active');
    });
    document.querySelectorAll('.view-section').forEach(sec => sec.classList.remove('active-view'));
    document.getElementById(`view-${viewName}`).classList.add('active-view');
    
    if (viewName === 'calendar') {
        updateRuleSelect();
        renderCalendar();
    }
}

/* --- Generator Logic --- */
function updateRuleSelect() {
    const select = document.getElementById('rule-select');
    if (!select) return; // エラー回避
    select.innerHTML = '';
    state.rules.forEach((rule, idx) => {
        const option = document.createElement('option');
        option.value = idx;
        option.textContent = rule.name;
        select.appendChild(option);
    });
}

function resolveStaffId(idStr) {
    if (!idStr) return null;
    const gPrefix = idStr.charAt(0);
    const sIdx = parseInt(idStr.substring(1));
    const gIdx = gPrefix.charCodeAt(0) - 97;

    const group = state.staffGroups[gIdx];
    if (!group || group.slots[sIdx] === undefined) {
        return { name: idStr + "?", groupIdx: 99 };
    }
    
    const memo = group.slots[sIdx];
    const dispName = memo ? memo : `${group.name}-${sIdx}`;
    
    return { name: dispName, groupIdx: gIdx };
}

function generateSchedule() {
    const select = document.getElementById('rule-select');
    const ruleIdx = select ? select.value : "";
    
    if (ruleIdx === "") { alert("Please define a rule first."); return; }
    
    const rule = state.rules[ruleIdx];
    const { year, month } = state;
    const daysInMonth = new Date(year, month + 1, 0).getDate();

    if (!confirm(`Apply rule "${rule.name}" to ${year}/${month + 1}? \nExisting data for this month will be overwritten.`)) return;

    for (let d = 1; d <= daysInMonth; d++) {
        const date = new Date(year, month, d);
        let dayIndex = date.getDay() - 1; 
        if (dayIndex === -1) dayIndex = 6; 
        
        const dayKey = days[dayIndex];
        const ruleDayData = rule.schedule[dayKey];

        const dateStr = `${year}-${String(month + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
        
        const m_staff = ruleDayData.m.map(resolveStaffId).filter(s => s);
        const a_staff = ruleDayData.a.map(resolveStaffId).filter(s => s);

        state.scheduleData[dateStr] = { m: m_staff, a: a_staff };
    }

    renderCalendar();
}

/* --- Config CRUD Actions --- */
function addNewGroup() { state.staffGroups.push({ name: `Group${state.staffGroups.length + 1}`, slots: ["", ""] }); renderConfig(); }
function removeGroup(i) { if(confirm("Shift IDs?")) { state.staffGroups.splice(i, 1); renderConfig(); } }
function updateGroupName(i, v) { state.staffGroups[i].name = v; renderConfig(); }

function addSlot(i) { state.staffGroups[i].slots.push(""); renderConfig(); }
function removeSlot(g, s) { state.staffGroups[g].slots.splice(s, 1); renderConfig(); }
function updateSlotMemo(g, s, v) { state.staffGroups[g].slots[s] = v; renderJSON(); }

function addNewRule() { const s = {}; days.forEach(d=>s[d]={m:[],a:[]}); state.rules.push({name:`rule${state.rules.length}`, schedule:s}); renderConfig(); }
function removeRule(i) { if(confirm("Del?")) { state.rules.splice(i, 1); renderConfig(); } }
function updateRuleName(i, v) { state.rules[i].name = v; renderJSON(); updateRuleSelect(); }
function removeAssignment(r, d, s, i) { state.rules[r].schedule[d][s].splice(i, 1); renderConfig(); }

/* --- Modal Actions --- */
function openModal(rIdx, day, shift) {
    const modalEl = document.getElementById('modal');
    const modalListEl = document.getElementById('modal-list');

    modalCtx = { rIdx, day, shift };
    modalListEl.replaceChildren();

    state.staffGroups.forEach((group, gIdx) => {
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
                onclick: () => confirmAssignment(idStr) 
            }, label));
        });
        
        container.appendChild(grid);
        modalListEl.appendChild(container);
    });
    
    modalEl.style.display = 'flex';
}

function confirmAssignment(idStr) {
    state.rules[modalCtx.rIdx].schedule[modalCtx.day][modalCtx.shift].push(idStr);
    document.getElementById('modal').style.display = 'none';
    renderConfig();
}

function closeModal() {
    document.getElementById('modal').style.display = 'none';
}


/* ==========================================================================
   4. RENDER FUNCTIONS (描画関数)
   ========================================================================== */

function renderCalendar() {
    const mount = document.getElementById('calendar-mount');
    const label = document.getElementById('current-month-label');
    const { year, month } = state;
    
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
        
        const cellContent = [];
        cellContent.push(el('span', { className: 'date-label' }, d));

        ['m', 'a'].forEach(shift => {
            if (dayData[shift] && dayData[shift].length > 0) {
                const chips = dayData[shift].map(s => el('span', { 
                    className: 'staff-chip', 
                    style:{borderLeftColor:getGroupColor(s.groupIdx)}, title: s.name 
                }, s.name));
                cellContent.push(el('div', { className: 'shift-section' }, 
                    el('div', { className: 'shift-label' }, shift === 'm' ? 'AM' : 'PM'), ...chips
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

function renderConfig() { 
    renderGroups(); 
    renderRules(); 
    renderJSON(); 
    updateRuleSelect(); 
}

function renderGroups() {
    const container = document.getElementById('staff-groups-container');
    container.replaceChildren();
    state.staffGroups.forEach((group, gIdx) => {
        const prefix = getGroupPrefix(gIdx);
        const color = getGroupColor(gIdx);
        const slotListContainer = el('div', { className: 'slot-list' });
        
        group.slots.forEach((memo, sIdx) => {
            slotListContainer.appendChild(el('div', { className: 'slot-item' },
                el('span', { className: 'slot-idx' }, `${sIdx}:`),
                el('input', { type: 'text', className: 'slot-input', value: memo, placeholder: 'Memo', oninput: (e) => updateSlotMemo(gIdx, sIdx, e.target.value) }),
                el('button', { className: 'btn btn-danger btn-sm', onclick: () => removeSlot(gIdx, sIdx) }, '×')
            ));
        });
        
        container.appendChild(el('div', { className: 'group-card', style: { borderTopColor: color } },
            el('div', { className: 'group-header' },
                el('span', { className: 'group-id-badge', style: { backgroundColor: color } }, `ID: ${prefix}`),
                el('button', { className: 'btn btn-danger btn-sm', onclick: () => removeGroup(gIdx) }, 'Delete')
            ),
            el('input', { type: 'text', className: 'group-name-input', value: group.name, placeholder: 'Group Name', oninput: (e) => updateGroupName(gIdx, e.target.value) }),
            slotListContainer,
            el('button', { className: 'btn btn-outline', style: { width: '100%', fontSize: '0.8em' }, onclick: () => addSlot(gIdx) }, '+ Add Slot')
        ));
    });
}

function renderRules() {
    const container = document.getElementById('rules-container');
    container.replaceChildren();
    
    state.rules.forEach((rule, rIdx) => {
        const theadTr = el('tr', {}, el('th', { className: 'config-row-header' }, 'Shift'));
        days.forEach(d => theadTr.appendChild(el('th', {}, d.toUpperCase())));
        
        const tbody = el('tbody');
        ['m', 'a'].forEach(shiftType => {
            const tr = el('tr', {});
            tr.appendChild(el('td', { className: 'config-row-header' }, shiftType === 'm' ? 'Morning' : 'Afternoon'));
            
            days.forEach(day => {
                const cell = el('td', {});
                rule.schedule[day][shiftType].forEach((idStr, arrIdx) => {
                    const gPrefix = idStr.charAt(0);
                    const gIdx = gPrefix.charCodeAt(0) - 97;
                    const color = getGroupColor(gIdx);
                    
                    // Config画面では ID (a0, b1) を表示
                    const label = idStr; 
                    
                    cell.appendChild(el('span', { 
                        className: 'chip', 
                        style: { backgroundColor: color }, 
                        title: idStr, 
                        onclick: () => removeAssignment(rIdx, day, shiftType, arrIdx) 
                    }, label));
                });
                cell.appendChild(el('button', { className: 'add-btn-mini', onclick: () => openModal(rIdx, day, shiftType) }, '+'));
                tr.appendChild(cell);
            });
            tbody.appendChild(tr);
        });
        
        container.appendChild(el('div', { className: 'rule-card' },
            el('div', { className: 'rule-header' },
                el('input', { type: 'text', style: { fontSize: '1.1em', fontWeight: 'bold' }, value: rule.name, oninput: (e) => updateRuleName(rIdx, e.target.value) }),
                el('button', { className: 'btn btn-danger', onclick: () => removeRule(rIdx) }, 'Delete Rule')
            ),
            el('table', { className: 'config-table' }, el('thead', {}, theadTr), tbody)
        ));
    });
}

function renderJSON() { 
    document.getElementById('json-output').textContent = JSON.stringify({staffGroups: state.staffGroups, rules: state.rules}, null, 2); 
}


/* ==========================================================================
   5. INITIALIZATION & EVENT LISTENERS (初期化とイベント設定)
   ========================================================================== */

function initApp() {
    // Calendar Controls
    document.getElementById('prev-btn').onclick = () => {
        state.month--;
        if (state.month < 0) { state.month = 11; state.year--; }
        renderCalendar();
    };
    
    document.getElementById('next-btn').onclick = () => {
        state.month++;
        if (state.month > 11) { state.month = 0; state.year++; }
        renderCalendar();
    };

    document.getElementById('generate-btn').onclick = generateSchedule;

    // Config Controls
    document.getElementById('add-group-btn').onclick = addNewGroup;
    document.getElementById('add-rule-btn').onclick = addNewRule;

    // Modal Controls
    document.getElementById('modal-cancel-btn').onclick = closeModal;
    document.getElementById('modal').onclick = (e) => { 
        if(e.target.id === 'modal') closeModal(); 
    };

    // Initial Render
    renderConfig();
    renderCalendar();
}

// Start the App
initApp();
