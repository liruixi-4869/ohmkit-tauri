const { invoke } = window.__TAURI__.core;

// ─── 标签切换 ───
document.querySelectorAll('#tab-nav .tab').forEach(t => {
  t.addEventListener('click', () => {
    document.querySelectorAll('#tab-nav .tab').forEach(x => x.classList.remove('active'));
    document.querySelectorAll('.panel').forEach(x => x.classList.remove('active'));
    t.classList.add('active');
    document.getElementById('panel-' + t.dataset.tab).classList.add('active');
  });
});

// ─── 表达式 ───
const exprInput = document.getElementById('expr-input');
const exprResult = document.getElementById('expr-result');

async function calcExpr() {
  const input = exprInput.value.trim();
  if (!input) return;
  try {
    const r = await invoke('calc_expr', { expr: input });
    if (r.error) { exprResult.innerHTML = `<span style="color:var(--red)">错误: ${r.error}</span>`; }
    else { exprResult.textContent = `${input}\n等效电阻 = ${r.value}`; }
  } catch (e) {
    exprResult.innerHTML = `<span style="color:var(--red)">调用失败: ${e}</span>`;
  }
}

document.getElementById('btn-calc-expr').addEventListener('click', calcExpr);
exprInput.addEventListener('keydown', e => { if (e.key === 'Enter') calcExpr(); });

document.querySelectorAll('#quick-btns button').forEach(b => {
  b.addEventListener('click', () => {
    const sym = b.textContent;
    const start = exprInput.selectionStart;
    exprInput.value = exprInput.value.slice(0, start) + sym + exprInput.value.slice(exprInput.selectionEnd);
    exprInput.focus();
    exprInput.setSelectionRange(start + sym.length, start + sym.length);
  });
});

document.querySelectorAll('#examples a').forEach(a => {
  a.addEventListener('click', () => {
    exprInput.value = a.textContent;
    calcExpr();
  });
});

// ─── Δ-Y ───
const d23 = document.getElementById('d23');
const d13 = document.getElementById('d13');
const d12 = document.getElementById('d12');
const yy1 = document.getElementById('y1');
const yy2 = document.getElementById('y2');
const yy3 = document.getElementById('y3');
const dyResult = document.getElementById('delta-result');

document.getElementById('btn-delta').addEventListener('click', async () => {
  try {
    const r = await invoke('calc_delta', { r23: d23.value, r13: d13.value, r12: d12.value });
    dyResult.textContent =
      `Δ (R23=${d23.value}, R13=${d13.value}, R12=${d12.value}) → Y\n` +
      `  R1 = ${r.r1}\n  R2 = ${r.r2}\n  R3 = ${r.r3}`;
  } catch(e) { dyResult.textContent = `错误: ${e}`; }
});

document.getElementById('btn-wye').addEventListener('click', async () => {
  try {
    const r = await invoke('calc_wye', { r1: yy1.value, r2: yy2.value, r3: yy3.value });
    dyResult.textContent =
      `Y (R1=${yy1.value}, R2=${yy2.value}, R3=${yy3.value}) → Δ\n` +
      `  R23 = ${r.r1}\n  R13 = ${r.r2}\n  R12 = ${r.r3}`;
  } catch(e) { dyResult.textContent = `错误: ${e}`; }
});

// ─── 桥式 ───
const br = ['br1','br2','br3','br4','br5'].map(id => document.getElementById(id));
const brResult = document.getElementById('bridge-result');

document.getElementById('btn-bridge').addEventListener('click', async () => {
  try {
    const r = await invoke('calc_bridge', {
      r1: br[0].value, r2: br[1].value, r3: br[2].value, r4: br[3].value, r5: br[4].value
    });
    brResult.textContent =
      `桥式  R1=${br[0].value} R2=${br[1].value} R3=${br[2].value} R4=${br[3].value} R5=${br[4].value}\n\n` +
      `Δ(R12=R5, R51=R2, R52=R1) → Y:\n` +
      `\n` +
      `               ┌── R2 ── C ── R3 ──┐\n` +
      `   A ── R5 ── O                   ├── B\n` +
      `               └── R1 ── D ── R4 ──┘\n` +
      `\n` +
      `           R5 = ${r.r2y}\n` +
      `           R2 = ${r.r1y}  R3 = ${br[2].value}\n` +
      `           R1 = ${r.r3y}  R4 = ${br[3].value}\n\n` +
      `等效电阻 = ${r.total}`;
  } catch(e) { brResult.textContent = `错误: ${e}`; }
});

// ─── 色环 ───
const colorVal = document.getElementById('color-val');
const colorResult = document.getElementById('color-result');

document.getElementById('btn-encode').addEventListener('click', async () => {
  if (!colorVal.value) return;
  try {
    const r = await invoke('encode_color', { value: colorVal.value });
    colorResult.textContent = r;
  } catch(e) { colorResult.textContent = `错误: ${e}`; }
});

document.getElementById('btn-decode').addEventListener('click', () => {
  const bands = document.getElementById('color-bands').value.trim();
  if (!bands) return;
  colorResult.textContent = '色环解码请使用 CLI: ohmkit color ' + bands;
});
