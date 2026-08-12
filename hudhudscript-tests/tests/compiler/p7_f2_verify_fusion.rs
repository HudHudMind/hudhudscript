use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use std::fs;

#[test]
fn verify_intmodcmpi_in_bytecode() {
    let fixture_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    for (name, file) in [("collatz", "collatz_conjecture.hhs")] {
        let path = format!("{}/{}", fixture_dir, file);
        let src = fs::read_to_string(&path).unwrap();
        let ast = parse(&src).unwrap();
        let mut compiler = Compiler::new();
        let bc = compiler.compile(&ast).unwrap();
        let mut has_modcmpi = false;
        let mut has_adjacent = false;

        // Scan top-level instructions
        for i in 0..bc.instructions.len().saturating_sub(1) {
            if matches!(bc.instructions[i], Instruction::IntModCmpI { .. }) {
                has_modcmpi = true;
            }
            if let (Instruction::IntModI { .. }, Instruction::IntCmpI { .. }) =
                (&bc.instructions[i], &bc.instructions[i + 1])
            {
                has_adjacent = true;
            }
        }
        // Also scan function bodies (collatz loop is inside collatz(n))
        let funcs = bc.functions.borrow();
        for chunk in funcs.iter() {
            for i in 0..chunk.instructions.len().saturating_sub(1) {
                if matches!(chunk.instructions[i], Instruction::IntModCmpI { .. }) {
                    has_modcmpi = true;
                }
                if let (Instruction::IntModI { .. }, Instruction::IntCmpI { .. }) =
                    (&chunk.instructions[i], &chunk.instructions[i + 1])
                {
                    has_adjacent = true;
                }
            }
        }
        println!(
            "{}: IntModCmpI={} adjacent_IntModI_CmpI={}",
            name, has_modcmpi, has_adjacent
        );
        assert!(has_modcmpi, "{} should have IntModCmpI", name);
        assert!(
            !has_adjacent,
            "{} should NOT have adjacent IntModI->IntCmpI",
            name
        );
    }
}
