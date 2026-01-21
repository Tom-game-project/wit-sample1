import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import type { 
    Plan, PlanConfig, StaffGroupWithMembers, WeeklyRuleWithAssignments, 
    ShiftCalendarManager, WeekStatus, RuleAssignment
} from "./types";

/* ==========================================================================
   STATE
   ========================================================================== */
let currentPlanId: number | null = null;
let currentConfig: PlanConfig | null = null;
let currentYear = new Date().getFullYear();
let currentMonth = new Date().getMonth();

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

    groups.forEach(g => {
        const div = document.createElement('div');
        div.className = 'group-card';
        div.style.border = '1px solid #ccc';
        div.style.padding = '10px';
        div.style.marginBottom = '10px';

        div.innerHTML = `
            <div style="display:flex; justify-content:space-between;">
                <strong>${g.group.name}</strong>
                <div>
                    <button class="btn-sm" onclick="window.updateGroupName(${g.group.id})">Edit</button>
                    <button class="btn-sm btn-danger" onclick="window.removeGroup(${g.group.id})">Del</button>
                </div>
            </div>
            <div class="members-list" style="margin-top:5px;"></div>
        `;
        
        const list = div.querySelector('.members-list')!;
        g.members.forEach(m => {
            const mDiv = document.createElement('div');
            mDiv.innerHTML = `
                <span>${m.sort_order}: ${m.name}</span>
                <button class="btn-sm" onclick="window.removeMember(${m.id})">x</button>
            `;
            list.appendChild(mDiv);
        });

        const addBtn = document.createElement('button');
        addBtn.className = "btn-sm btn-outline";
        addBtn.textContent = "+ Add Member";
        addBtn.onclick = () => addMember(g.group.id);
        div.appendChild(addBtn);

        container.appendChild(div);
    });
}

// Rules Logic
function renderRules(rules: WeeklyRuleWithAssignments[]) {
    const container = document.getElementById('rules-container');
    if (!container) return;
    container.innerHTML = '';

    rules.forEach(r => {
        const div = document.createElement('div');
        div.className = 'rule-card';
        div.style.border = '1px solid #ccc';
        div.style.padding = '10px';
        div.style.marginBottom = '10px';

        div.innerHTML = `
            <div style="display:flex; justify-content:space-between;">
                <strong>${r.rule.name}</strong>
                <button class="btn-sm btn-danger" onclick="window.removeRule(${r.rule.id})">Del</button>
            </div>
            <div style="font-size:0.8em; margin-top:5px;">
                Assignments: ${r.assignments.length}
            </div>
        `;
        // TODO: アサインメントの追加・削除用モーダル連携はここに実装
        const addAssignBtn = document.createElement('button');
        addAssignBtn.textContent = "Add Assign (Dummy)";
        addAssignBtn.onclick = () => {
             // 実際はモーダルを開いて GroupとIndexを選択させる
             addAssignment(r.rule.id, 0, 0, currentConfig!.groups[0].group.id, 0);
        };
        div.appendChild(addAssignBtn);

        container.appendChild(div);
    });
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

/* ==========================================================================
   CALENDAR VIEW
   ========================================================================== */
async function renderCalendarView() {
    if (!currentPlanId) return;
    
    const label = document.getElementById('current-month-label');
    if (label) label.textContent = new Date(currentYear, currentMonth, 1).toLocaleDateString('ja-JP', { year: 'numeric', month: 'long' });

    const mount = document.getElementById('calendar-mount');
    if(!mount) return;
    mount.innerHTML = 'Loading...';

    const weeksData = calculateCalendarDates(currentYear, currentMonth);

    // 状態取得
    let savedManager: ShiftCalendarManager | null = null;
    try {
        savedManager = await invoke<ShiftCalendarManager>("get_calendar_state", { planId: currentPlanId });
    } catch(e) {}

    mount.innerHTML = '';
    weeksData.forEach((week, i) => {
        const row = document.createElement('div');
        row.className = 'cal-week-row';
        row.style.display = 'flex';
        row.style.borderBottom = '1px solid #eee';

        // 簡易表示: ステータス
        const control = document.createElement('div');
        control.style.width = '100px';
        control.style.padding = '5px';
        control.textContent = `Week ${i+1}`;
        row.appendChild(control);

        week.days.forEach(day => {
            const cell = document.createElement('div');
            cell.style.flex = '1';
            cell.style.borderLeft = '1px solid #eee';
            cell.style.padding = '5px';
            cell.textContent = day.getDate().toString();
            if (day.getMonth() !== currentMonth) cell.style.color = '#ccc';
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
}

// Global Exports for onclick
(window as any).removeGroup = removeGroup;
(window as any).updateGroupName = updateGroupName;
(window as any).removeMember = removeMember;
(window as any).removeRule = removeRule;
