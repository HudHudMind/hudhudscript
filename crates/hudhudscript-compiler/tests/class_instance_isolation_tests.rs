use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn test_multiple_class_instances_isolation() {
    let src = r#"
class TaskData {
    public function constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    public fn getX() {
        return this.x;
    }

    public fn setX(number) {
        this.x = number;
    }

    public fn getY() {
        return this.y;
    }

    public fn setY(number) {
        this.y = number;
    }
}

let taskDataObj1 = new TaskData(1, 99);
let x1_before = taskDataObj1.getX();
taskDataObj1.setX(99);
let x1_after = taskDataObj1.getX();
let y1 = taskDataObj1.getY();

let taskDataObj2 = new TaskData(0, 0);
let x2_before = taskDataObj2.getX();
taskDataObj2.setX(99);
let x2_after = taskDataObj2.getX();
let y2 = taskDataObj2.getY();

let taskDataObj3 = new TaskData(3, 4);
let x3_before = taskDataObj3.getX();
taskDataObj3.setX(99);
let x3_after = taskDataObj3.getX();
let y3 = taskDataObj3.getY();
"#;

    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    let res = vm.execute(&bc);
    assert!(res.is_ok(), "VM execution failed: {:?}", res);

    // Obj1 assertions
    assert_eq!(vm.get_variable("x1_before").unwrap().as_int(), Some(1));
    assert_eq!(vm.get_variable("x1_after").unwrap().as_int(), Some(99));
    assert_eq!(vm.get_variable("y1").unwrap().as_int(), Some(99));

    // Obj2 assertions (must start with 0, 0, NOT leak obj1's 99!)
    assert_eq!(vm.get_variable("x2_before").unwrap().as_int(), Some(0));
    assert_eq!(vm.get_variable("x2_after").unwrap().as_int(), Some(99));
    assert_eq!(vm.get_variable("y2").unwrap().as_int(), Some(0));

    // Obj3 assertions (must start with 3, 4, NOT leak obj2's 99!)
    assert_eq!(vm.get_variable("x3_before").unwrap().as_int(), Some(3));
    assert_eq!(vm.get_variable("x3_after").unwrap().as_int(), Some(99));
    assert_eq!(vm.get_variable("y3").unwrap().as_int(), Some(4));
}

#[test]
fn test_class_constructor_independent_fields() {
    let src = r#"
class Point {
    public function constructor(px, py) {
        this.px = px;
        this.py = py;
    }
    public fn getSum() {
        return this.px + this.py;
    }
}

let p1 = new Point(10, 20);
let p2 = new Point(100, 200);
let s1 = p1.getSum();
let s2 = p2.getSum();
"#;

    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    let res = vm.execute(&bc);
    assert!(res.is_ok(), "VM execution failed: {:?}", res);

    assert_eq!(vm.get_variable("s1").unwrap().as_int(), Some(30));
    assert_eq!(vm.get_variable("s2").unwrap().as_int(), Some(300));
}
