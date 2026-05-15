use std::fmt;
use tauri::command;

// ─── 分数引擎 ───

#[derive(Debug, Clone, Copy, serde::Serialize)]
struct Frac {
    num: i64,
    den: i64,
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs(); b = b.abs();
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

impl Frac {
    fn new(num: i64, den: i64) -> Self {
        if den == 0 { return Self { num: 0, den: 1 }; }
        let sign = if den < 0 { -1 } else { 1 };
        let g = gcd(num, den);
        Self { num: sign * num / g, den: (sign * den / g).abs().max(1) }
    }

    fn from_int(n: i64) -> Self { Self::new(n, 1) }

    fn add(self, other: Self) -> Self {
        Self::new(self.num * other.den + other.num * self.den, self.den * other.den)
    }

    fn parallel(self, other: Self) -> Self {
        if self.num == 0 || other.num == 0 { return Self::new(0, 1); }
        Self::new(self.num * other.num, self.num * other.den + other.num * self.den)
    }

    fn to_f64(self) -> f64 { self.num as f64 / self.den as f64 }
}

impl fmt::Display for Frac {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.den == 1 { write!(f, "{}", self.num) }
        else { write!(f, "{}/{}", self.num, self.den) }
    }
}

fn fmt_ohm(f: Frac) -> String {
    let v = f.to_f64();
    if v >= 1e9 { format!("{} (≈ {:.4} GΩ)", f, v/1e9) }
    else if v >= 1e6 { format!("{} (≈ {:.4} MΩ)", f, v/1e6) }
    else if v >= 1e3 { format!("{} (≈ {:.4} kΩ)", f, v/1e3) }
    else { format!("{} (≈ {:.4} Ω)", f, v) }
}

// ─── 表达式解析 ───

#[derive(Debug, serde::Serialize)]
struct ExprResult { value: String, error: Option<String> }

fn parse_expr_str(s: &str) -> ExprResult {
    let s = s.trim();
    if s.is_empty() {
        return ExprResult { value: String::new(), error: Some("表达式为空".into()) };
    }
    match parse_expr(s.as_bytes(), 0) {
        Ok((val, pos)) => {
            let rest = s[pos..].trim();
            if rest.is_empty() {
                ExprResult { value: fmt_ohm(val), error: None }
            } else {
                ExprResult { value: fmt_ohm(val), error: Some(format!("'{}' 无法解析", rest)) }
            }
        }
        Err(e) => ExprResult { value: String::new(), error: Some(e) },
    }
}

fn parse_expr(s: &[u8], pos: usize) -> Result<(Frac, usize), String> {
    let (mut val, mut pos) = parse_term(s, pos)?;
    loop {
        pos = skip_space(s, pos);
        if pos < s.len() && s[pos] == b'+' {
            pos += 1;
            let (rhs, p) = parse_term(s, pos)?;
            val = val.add(rhs);
            pos = p;
        } else { break; }
    }
    Ok((val, pos))
}

fn parse_term(s: &[u8], pos: usize) -> Result<(Frac, usize), String> {
    let (mut val, mut pos) = parse_atom(s, pos)?;
    loop {
        pos = skip_space(s, pos);
        if pos + 1 < s.len() && s[pos] == b'|' && s[pos+1] == b'|' {
            pos += 2;
            let (rhs, p) = parse_atom(s, pos)?;
            val = val.parallel(rhs);
            pos = p;
        } else { break; }
    }
    Ok((val, pos))
}

fn parse_atom(s: &[u8], pos: usize) -> Result<(Frac, usize), String> {
    let pos = skip_space(s, pos);
    if pos >= s.len() { return Err("表达式不完整".into()); }
    if s[pos] == b'(' {
        let (val, pos) = parse_expr(s, pos + 1)?;
        let pos = skip_space(s, pos);
        if pos >= s.len() || s[pos] != b')' { return Err("缺少 ')'".into()); }
        return Ok((val, pos + 1));
    }
    if s[pos].is_ascii_digit() || s[pos] == b'.' || s[pos] == b'-' {
        let start = pos;
        let mut end = pos;
        if s[end] == b'-' { end += 1; }
        while end < s.len() && (s[end].is_ascii_digit() || s[end] == b'.') { end += 1; }
        let num_str = std::str::from_utf8(&s[start..end]).map_err(|_| "无效数字".to_string())?;
        let fv: f64 = num_str.parse().map_err(|_| format!("无效数字: {}", num_str))?;
        let mut pos = end;

        /* 支持分数输入: 3/7 */
        pos = skip_space(s, pos);
        if pos < s.len() && s[pos] == b'/' {
            pos += 1;
            pos = skip_space(s, pos);
            let d_start = pos;
            while pos < s.len() && s[pos].is_ascii_digit() { pos += 1; }
            let den: i64 = std::str::from_utf8(&s[d_start..pos])
                .map_err(|_| "无效分母".to_string())?
                .parse().map_err(|_| "无效分母".to_string())?;
            if den == 0 { return Err("分母不能为0".into()); }
            return Ok((Frac::new(fv as i64, den), pos));
        }

        return float_to_frac(fv).map(|f| (f, pos));
    }
    Err(format!("意外字符 '{}'", s[pos] as char))
}

fn float_to_frac(fv: f64) -> Result<Frac, String> {
    if fv.fract().abs() < 1e-12 {
        return Ok(Frac::from_int(fv as i64));
    }
    let den = 1_000_000_000i64;
    let num = (fv * den as f64).round() as i64;
    Ok(Frac::new(num, den))
}

fn skip_space(s: &[u8], pos: usize) -> usize {
    let mut p = pos;
    while p < s.len() && (s[p] == b' ' || s[p] == b'\t') { p += 1; }
    p
}

// ─── Δ-Y 变换 ───

#[derive(Debug, serde::Serialize)]
struct DeltaWyeResult { r1: String, r2: String, r3: String }

fn delta_to_wye(r12: Frac, r23: Frac, r13: Frac) -> (Frac, Frac, Frac) {
    let sum = r12.add(r23).add(r13);
    if sum.num == 0 { return (Frac::from_int(0), Frac::from_int(0), Frac::from_int(0)); }
    let r1 = Frac::new(r12.num * r13.num, r12.den * r13.den);
    let r1 = Frac::new(r1.num * sum.den, r1.den * sum.num);
    let r2 = Frac::new(r12.num * r23.num, r12.den * r23.den);
    let r2 = Frac::new(r2.num * sum.den, r2.den * sum.num);
    let r3 = Frac::new(r23.num * r13.num, r23.den * r13.den);
    let r3 = Frac::new(r3.num * sum.den, r3.den * sum.num);
    (r1, r2, r3)
}

fn wye_to_delta(r1: Frac, r2: Frac, r3: Frac) -> (Frac, Frac, Frac) {
    let _n = r1.add(r2).add(r3); // dummy, need product sum
    let n = Frac::new(
        r1.num*r2.num*r3.den + r2.num*r3.num*r1.den + r3.num*r1.num*r2.den,
        r1.den*r2.den*r3.den
    );
    let n = Frac::new(n.num, n.den);
    let r12 = if r3.num == 0 { Frac::from_int(0) }
              else { Frac::new(n.num * r3.den, n.den * r3.num) };
    let r23 = if r1.num == 0 { Frac::from_int(0) }
              else { Frac::new(n.num * r1.den, n.den * r1.num) };
    let r13 = if r2.num == 0 { Frac::from_int(0) }
              else { Frac::new(n.num * r2.den, n.den * r2.num) };
    (r12, r23, r13)
}

// ─── 色环 ───

const DIGIT_EN: [&str; 10] = ["black","brown","red","orange","yellow","green","blue","violet","grey","white"];
const DIGIT_CN: [&str; 10] = ["黑","棕","红","橙","黄","绿","蓝","紫","灰","白"];
const E24: [i32; 24] = [10,11,12,13,15,16,18,20,22,24,27,30,33,36,39,43,47,51,56,62,68,75,82,91];

// ─── Tauri commands ───

#[command]
fn calc_expr(expr: String) -> ExprResult { parse_expr_str(&expr) }

#[command]
fn calc_delta(r23: String, r13: String, r12: String) -> DeltaWyeResult {
    let r12 = float_to_frac(r12.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r23 = float_to_frac(r23.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r13 = float_to_frac(r13.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let (r1, r2, r3) = delta_to_wye(r12, r23, r13);
    DeltaWyeResult { r1: fmt_ohm(r1), r2: fmt_ohm(r2), r3: fmt_ohm(r3) }
}

#[command]
fn calc_wye(r1: String, r2: String, r3: String) -> DeltaWyeResult {
    let r1 = float_to_frac(r1.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r2 = float_to_frac(r2.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r3 = float_to_frac(r3.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let (r12, r23, r13) = wye_to_delta(r1, r2, r3);
    DeltaWyeResult { r1: fmt_ohm(r23), r2: fmt_ohm(r13), r3: fmt_ohm(r12) }
}

#[derive(Debug, serde::Serialize)]
struct BridgeResult {
    r1y: String, r2y: String, r3y: String, total: String,
}

#[command]
fn calc_bridge(r1: String, r2: String, r3: String, r4: String, r5: String) -> BridgeResult {
    let r1 = float_to_frac(r1.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r2 = float_to_frac(r2.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r3 = float_to_frac(r3.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r4 = float_to_frac(r4.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let r5 = float_to_frac(r5.parse().unwrap_or(0.0)).unwrap_or(Frac::from_int(0));
    let (r1y, r2y, r3y) = delta_to_wye(r1, r2, r5);
    let path_top = r1y.add(r3);
    let path_bot = r3y.add(r4);
    let mid = path_top.parallel(path_bot);
    let total = r2y.add(mid);
    BridgeResult { r1y: fmt_ohm(r1y), r2y: fmt_ohm(r2y), r3y: fmt_ohm(r3y), total: fmt_ohm(total) }
}

#[command]
fn encode_color(value: String) -> String {
    let val: f64 = value.parse().unwrap_or(0.0);
    let absv = val.abs();
    if absv < 1e-12 { return "0 Ω = 黑 黑 黑".into(); }
    let mut mant = absv;
    let mut exp10: i32 = 0;
    while mant >= 100.0 { mant /= 10.0; exp10 += 1; }
    while mant < 10.0   { mant *= 10.0; exp10 -= 1; }
    let best = E24.iter().min_by(|a, b| {
        (mant - **a as f64).abs().partial_cmp(&(mant - **b as f64).abs()).unwrap()
    }).unwrap();
    let d1 = best / 10;
    let d2 = best % 10;
    let mul = exp10;
    let (mul_en, mul_cn) = if mul >= 0 { (DIGIT_EN[mul as usize], DIGIT_CN[mul as usize]) }
        else if mul == -1 { ("gold", "金") }
        else { ("silver", "银") };
    let result = *best as f64 * 10f64.powi(mul);
    format!("{} {} {} gold  →  {} {} {} 金  =  {:.4} Ω ±5%",
        DIGIT_EN[d1 as usize], DIGIT_EN[d2 as usize], mul_en,
        DIGIT_CN[d1 as usize], DIGIT_CN[d2 as usize], mul_cn, result)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            calc_expr, calc_delta, calc_wye, calc_bridge, encode_color
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
