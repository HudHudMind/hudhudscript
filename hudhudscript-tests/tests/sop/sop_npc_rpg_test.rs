//! SOP doğrulama testi (moved from crates/hudhudscript-test/tests)

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn sop_spawn_and_print() {
    let source = r#"
subject Boss {
    state health: 200,
    state stamina: 100
}
let hero = spawn Boss
print(hero.health)
print(hero.stamina)
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
}

#[test]
fn sop_simple_effect() {
    let source = r#"
subject Boss {
    state health: 200
}
effect on Smash(target, dmg) {
    target.health = target.health - dmg
}
let boss = spawn Boss
Smash(boss, 30)
print(boss.health)
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
}

#[test]
fn sop_capability_call() {
    let source = r#"
subject Boss {
    state health: 200
}
on attack(self, target) {
    target.health = target.health - 30
}
let hero = spawn Boss
let boss = spawn Boss
hero.attack(boss)
print(boss.health)
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
}

#[test]
fn sop_npc_rpg_compiles() {
    let source = include_str!("../../../hudhud-script/examples/05-advanced/sop_npc_rpg.hud");
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let _bytecode = compiler.compile(&ast).expect("compile");
}

#[test]
fn sop_npc_rpg_executes() {
    // Full NPC RPG execute test — self.power, self.stamina, target.health all working
    let source = include_str!("../../../hudhud-script/examples/05-advanced/sop_npc_rpg.hud");
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode)
        .expect("NPC RPG execute should succeed");
}

#[test]
fn sop_subject_scoped_capability() {
    // Subject içinde capability tanımı + subject-scoped dispatch
    let source = r#"
subject Knight {
    state health: 100
    state power: 30
    on attack(self, target) {
        target.health = target.health - self.power
    }
}
subject Dragon {
    state health: 200
}
let hero = spawn Knight
let boss = spawn Dragon
hero.attack(boss)
print(boss.health)
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode)
        .expect("subject-scoped capability execute");
}

#[test]
fn sop_subject_scoped_top_level() {
    // Top-level subject-scoped capability: on Knight.attack(...)
    let source = r#"
subject Knight {
    state health: 100
}
subject Dragon {
    state health: 200
}
on Knight.attack(self, target) {
    target.health = target.health - 30
}
let hero = spawn Knight
let boss = spawn Dragon
hero.attack(boss)
print(boss.health)
"#;
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
}
