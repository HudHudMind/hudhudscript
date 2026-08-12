//! G12 — unboxed float döngü pass'i (exp/unboxed-float).
//!
//! Sıcak `while` gövdelerinde float-KANITLI yereller Value16 register'ı
//! yerine VM'in `f_slots: [f64; 64]` dosyasında yaşar; gövde içi aritmetik
//! tag'siz F-op'larla koşar. Döngü girişinde FLoadNum (kutu-aç), çıkışında
//! FStoreNum (kutula) köprüleri kurulur.
//!
//! Doğruluk modeli (Kural 7c — tek yol, fallback YOK):
//! - Aday yereller döngü boyunca YALNIZ f-slot'tan günceldir; register'ları
//!   bayat kalır. Bu yüzden gövde içinde adayın register'ını okuyabilecek
//!   HİÇBİR kullanım olmamalıdır — analiz bunu kanıtlamadan pass devreye
//!   girmez (sessiz yanlış-kod riski alınmaz; kanıtlanamayan döngü eski
//!   yoldan derlenir).
//! - Kaçış/istisna güvenliği: gövdede çağrı (Math.sin/cos/sqrt intrinsic'i
//!   hariç), index, property, try, throw, kapama (closure), iç içe döngü,
//!   break/continue, switch/match vs. varsa pass TÜMÜYLE devre dışı kalır.
//!   Böylece döngü ortasında unwinding ile epilogun atlanması imkânsızdır.

use super::floop_analyze::analyze;
use super::*;
use crate::compiler::expr::ExprType;
use hudhudscript_ast::UnaryOp;

/// Geçerli plan: aday slotları + prolog/epilog köprü bilgisi.
pub(super) struct FloopPlan {
    /// (isim, fslot, register) — döngü öncesi var olan adaylar.
    pre: Vec<(String, u8, u8)>,
    /// (isim, fslot) — tüm adaylar (gövde-içi let'ler dahil).
    slots: Vec<(String, u8)>,
    /// Geçici f-slot tabanı (adaya ait olmayan ilk slot).
    temp_base: u8,
}

impl FloopPlan {
    pub(super) fn new(
        pre: Vec<(String, u8, u8)>,
        slots: Vec<(String, u8)>,
        temp_base: u8,
    ) -> Self {
        Self { pre, slots, temp_base }
    }
}

/// f-ifade sınıfı: No = f-domain'e giremez, Int = tümü-int (Div'de
/// truncation semantiği yüzünden yasak), Float = float-kanıtlı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FE {
    No,
    Int,
    Float,
}

// ── Genel giriş/çıkış ─────────────────────────────────────────────

/// While kolunun başında çağrılır: analiz + sabit hoist'i + prolog + bağlam
/// kurulumu. Gövdedeki f-domain sabitleri döngü-değişmezidir: FConst'ları
/// prologda BİR KEZ koşar, gövde slotlardan okur (87→~45 komut/iterasyonun
/// ana kalemi FConst/FMove tekrarıydı).
pub(super) fn enter(
    target: &mut impl CompileTarget,
    condition: &Expr,
    body: &Stmt,
) -> Option<FloopPlan> {
    let plan = analyze(target, condition, body)?;
    // Sabit taraması: aday let/atama RHS'lerindeki tüm sayısal literaller
    // (+ Neg için -1.0). Kapasite aşımı = f-domain'e hiç girmeden eski yol
    // (henüz hiçbir şey emit edilmedi — tek karar noktası, fallback değil).
    let slot_names: HashSet<String> = plan.slots.iter().map(|(n, _)| n.clone()).collect();
    let mut const_bits: Vec<u64> = Vec::new();
    collect_f_consts(body, &slot_names, &mut const_bits);
    let n_vars = plan.temp_base as usize;
    if n_vars + const_bits.len() > 56 {
        return None;
    }
    let mut consts: Vec<(u64, u8)> = Vec::new();
    for (i, bits) in const_bits.iter().enumerate() {
        consts.push((*bits, (n_vars + i) as u8));
    }
    let temp_base = (n_vars + consts.len()) as u8;
    for (_name, fslot, reg) in &plan.pre {
        target.ct_emit(Instruction::FLoadNum { fslot: *fslot, src: *reg });
    }
    for (bits, slot) in &consts {
        let idx = target.ct_emit_num_const(f64::from_bits(*bits));
        target.ct_emit(Instruction::FConst { d: *slot, const_idx: idx as u16 });
    }
    target.ct_floop_push(plan.slots.clone(), consts, temp_base);
    Some(plan)
}

/// Aday let/atama RHS'lerindeki sayısal literal bit desenlerini toplar
/// (tekrarsız, ilk-görülme sırasıyla). Yalnız f-domain'e derlenecek
/// ifadeler gezilir — koşullar register-domain'de kalır, taranmaz.
fn collect_f_consts(stmt: &Stmt, slots: &HashSet<String>, out: &mut Vec<u64>) {
    match stmt {
        Stmt::Let { name, value, .. } => {
            if slots.contains(name.as_str()) {
                collect_expr_consts(value, out);
            }
        }
        Stmt::Assignment { target, value, .. } => {
            if let Expr::Identifier(t, _) = target {
                if slots.contains(t.as_str()) {
                    collect_expr_consts(value, out);
                }
            }
        }
        Stmt::If { then_branch, else_branch, .. } => {
            collect_f_consts(then_branch, slots, out);
            if let Some(e) = else_branch {
                collect_f_consts(e, slots, out);
            }
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_f_consts(s, slots, out);
            }
        }
        _ => {}
    }
}

fn collect_expr_consts(expr: &Expr, out: &mut Vec<u64>) {
    let mut push = |bits: u64, out: &mut Vec<u64>| {
        if !out.contains(&bits) {
            out.push(bits);
        }
    };
    match expr {
        Expr::Literal(Literal::Number(n, _), _) => push(n.to_bits(), out),
        Expr::Literal(Literal::Int(i), _) => push((*i as f64).to_bits(), out),
        Expr::Binary { left, right, .. } => {
            collect_expr_consts(left, out);
            collect_expr_consts(right, out);
        }
        Expr::Unary { op: UnaryOp::Neg, expr: inner, .. } => {
            // Neg, hoist edilmiş -1.0 ile FMul olarak emit edilir.
            push((-1.0f64).to_bits(), out);
            collect_expr_consts(inner, out);
        }
        Expr::Unary { op: UnaryOp::Plus, expr: inner, .. } => {
            collect_expr_consts(inner, out);
        }
        Expr::Call { args, .. } if math_intrinsic(expr).is_some() => {
            collect_expr_consts(&args[0], out);
        }
        _ => {}
    }
}

/// While kolunun sonunda çağrılır (o anki ip == döngü çıkışı `end`).
pub(super) fn exit(target: &mut impl CompileTarget, plan: Option<FloopPlan>) {
    if let Some(plan) = plan {
        target.ct_floop_pop();
        for (_name, fslot, reg) in &plan.pre {
            target.ct_emit(Instruction::FStoreNum { dst: *reg, fslot: *fslot });
        }
    }
}

/// `let name = value` — adaysa f-domain'e derler, true döner.
pub(super) fn compile_let(
    target: &mut impl CompileTarget,
    name: &str,
    value: &Expr,
) -> CompileResult<bool> {
    let Some(slot) = target.ct_floop_slot(name) else {
        return Ok(false);
    };
    emit_f(target, value, slot)?;
    target.ct_declare_local(name, false)?;
    target.ct_set_local_type(name, ExprType::Number);
    Ok(true)
}

/// `name = value` — adaysa f-domain'e derler, true döner.
pub(super) fn compile_assign(
    target: &mut impl CompileTarget,
    name: &str,
    value: &Expr,
) -> CompileResult<bool> {
    let Some(slot) = target.ct_floop_slot(name) else {
        return Ok(false);
    };
    emit_f(target, value, slot)?;
    target.ct_set_local_type(name, ExprType::Number);
    Ok(true)
}

// ── f-domain emit (3-adres) ───────────────────────────────────────

/// Operand referansı: `Fixed` = değişken/sabit slotu (SERBEST BIRAKILMAZ,
/// üzerine yazılamaz), `Temp` = bu ifadenin sahip olduğu geçici (kullanım
/// sonrası iade edilir). F-op'lar 3-adreslidir: operandlar slotlarından
/// DOĞRUDAN okunur — kopya FMove'u yalnız kök `x = y` şeklinde gerekir.
#[derive(Clone, Copy)]
enum FRef {
    Fixed(u8),
    Temp(u8),
}

impl FRef {
    fn slot(self) -> u8 {
        match self {
            FRef::Fixed(s) | FRef::Temp(s) => s,
        }
    }
}

fn f_bug(what: &str) -> hudhudscript_bytecode::error::CompileError {
    compile_codes::generic(format!(
        "G12 f-loop analiz/emit uyumsuzluğu (derleyici bug'ı): {}",
        what
    ))
}

/// Geçiciyse iade et (LIFO — çağıran, ayırma sırasının tersiyle çağırır).
fn f_free(target: &mut impl CompileTarget, r: FRef) {
    if matches!(r, FRef::Temp(_)) {
        target.ct_floop_temp_pop();
    }
}

/// İfadeyi operand olarak derler: değişken/sabit ise 0 komut (slotu döner),
/// bileşikse sonucu yeni bir geçiciye hesaplar. Tüm F-op'lar operandları
/// okuduktan SONRA yazdığı için d'nin a/b ile örtüşmesi güvenlidir.
fn emit_operand(target: &mut impl CompileTarget, expr: &Expr) -> CompileResult<FRef> {
    match expr {
        Expr::Identifier(name, _) => target
            .ct_floop_slot(name)
            .map(FRef::Fixed)
            .ok_or_else(|| f_bug("aday slotu yok")),
        Expr::Literal(Literal::Number(n, _), _) => target
            .ct_floop_const_slot(n.to_bits())
            .map(FRef::Fixed)
            .ok_or_else(|| f_bug("hoist edilmemiş sabit")),
        Expr::Literal(Literal::Int(i), _) => target
            .ct_floop_const_slot((*i as f64).to_bits())
            .map(FRef::Fixed)
            .ok_or_else(|| f_bug("hoist edilmemiş sabit")),
        Expr::Binary { left, op, right, .. }
            if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div) =>
        {
            let a = emit_operand(target, left)?;
            let b = emit_operand(target, right)?;
            f_free(target, b);
            f_free(target, a);
            let d = target.ct_floop_temp().ok_or_else(|| f_bug("geçici slot tükendi"))?;
            target.ct_emit(f_binop(op, d, a.slot(), b.slot()));
            Ok(FRef::Temp(d))
        }
        Expr::Unary { op: UnaryOp::Neg, expr: inner, .. } => {
            // -v yerine v * -1.0: işaretli-sıfır ve sonsuz semantiği birebir
            // (0.0 - v, +0.0 için -0.0 ÜRETMEZ — Neg eder). -1.0 hoist'li.
            let neg1 = target
                .ct_floop_const_slot((-1.0f64).to_bits())
                .ok_or_else(|| f_bug("hoist edilmemiş -1.0"))?;
            let a = emit_operand(target, inner)?;
            f_free(target, a);
            let d = target.ct_floop_temp().ok_or_else(|| f_bug("geçici slot tükendi"))?;
            target.ct_emit(Instruction::FMul { d, a: a.slot(), b: neg1 });
            Ok(FRef::Temp(d))
        }
        Expr::Unary { op: UnaryOp::Plus, expr: inner, .. } => emit_operand(target, inner),
        Expr::Call { args, .. } if math_intrinsic(expr).is_some() => {
            let property = math_intrinsic(expr).expect("az önce kontrol edildi");
            let a = emit_operand(target, &args[0])?;
            f_free(target, a);
            let d = target.ct_floop_temp().ok_or_else(|| f_bug("geçici slot tükendi"))?;
            target.ct_emit(f_intrinsic(property, d, a.slot()));
            Ok(FRef::Temp(d))
        }
        _ => Err(f_bug("f-domain dışı ifade")),
    }
}

fn f_binop(op: &BinaryOp, d: u8, a: u8, b: u8) -> Instruction {
    match op {
        BinaryOp::Add => Instruction::FAdd { d, a, b },
        BinaryOp::Sub => Instruction::FSub { d, a, b },
        BinaryOp::Mul => Instruction::FMul { d, a, b },
        _ => Instruction::FDiv { d, a, b },
    }
}

fn f_intrinsic(property: &str, d: u8, s: u8) -> Instruction {
    match property {
        "sin" => Instruction::FSin { d, s },
        "cos" => Instruction::FCos { d, s },
        _ => Instruction::FSqrt { d, s },
    }
}

/// İfadeyi `dst` f-slot'una derler (kök: let/atama hedefi). Kök işlem
/// operandlarını okuduktan sonra yazdığından `dst`'nin operandlarla
/// örtüşmesi (theta = theta + ...) güvenlidir. Analiz kanıtı dışında bir
/// şekille karşılaşırsa bu bir derleyici BUG'ıdır — sessizce eski yola
/// DÜŞMEZ (prolog çoktan emit edildi, register'lar bayat; düşmek
/// yanlış-kod olur).
fn emit_f(target: &mut impl CompileTarget, expr: &Expr, dst: u8) -> CompileResult<()> {
    match expr {
        Expr::Identifier(..) | Expr::Literal(..) => {
            let r = emit_operand(target, expr)?;
            if r.slot() != dst {
                target.ct_emit(Instruction::FMove { d: dst, s: r.slot() });
            }
        }
        Expr::Binary { left, op, right, .. }
            if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div) =>
        {
            let a = emit_operand(target, left)?;
            let b = emit_operand(target, right)?;
            f_free(target, b);
            f_free(target, a);
            target.ct_emit(f_binop(op, dst, a.slot(), b.slot()));
        }
        Expr::Unary { op: UnaryOp::Neg, expr: inner, .. } => {
            let neg1 = target
                .ct_floop_const_slot((-1.0f64).to_bits())
                .ok_or_else(|| f_bug("hoist edilmemiş -1.0"))?;
            let a = emit_operand(target, inner)?;
            f_free(target, a);
            target.ct_emit(Instruction::FMul { d: dst, a: a.slot(), b: neg1 });
        }
        Expr::Unary { op: UnaryOp::Plus, expr: inner, .. } => emit_f(target, inner, dst)?,
        Expr::Call { args, .. } if math_intrinsic(expr).is_some() => {
            let property = math_intrinsic(expr).expect("az önce kontrol edildi");
            let a = emit_operand(target, &args[0])?;
            f_free(target, a);
            target.ct_emit(f_intrinsic(property, dst, a.slot()));
        }
        _ => return Err(f_bug("f-domain dışı ifade")),
    }
    Ok(())
}

/// Math.sin/cos/sqrt tek-argüman intrinsic şekli — compile_complex'teki
/// G8 deseniyle birebir aynı koşullar (gölge kontrolü çağıranın işi).
pub(super) fn math_intrinsic(expr: &Expr) -> Option<&str> {
    if let Expr::Call { callee, args, .. } = expr {
        if args.len() == 1 && !matches!(args[0], Expr::Spread { .. }) {
            if let Expr::Member { object, property, .. } = callee.as_ref() {
                if matches!(property.as_str(), "sin" | "cos" | "sqrt")
                    && matches!(object.as_ref(), Expr::Identifier(n, _) if n == "Math")
                {
                    return Some(match property.as_str() {
                        "sin" => "sin",
                        "cos" => "cos",
                        _ => "sqrt",
                    });
                }
            }
        }
    }
    None
}
