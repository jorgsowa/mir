// crates/mir-analyzer/tests/undefined_method.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_missing_instance_method() {
    // Call a method that does not exist on the class
    // line 5: "    $f->missing();" — col 4 ($f starts at col 4)
    let src = "<?php\nclass Foo {}\nfunction test(): void {\n    $f = new Foo();\n    $f->missing();\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedMethod", 5, 4);
}

#[test]
fn does_not_report_defined_method() {
    let src = "<?php\nclass Foo {\n    public function bar(): void {}\n}\nfunction test(): void {\n    $f = new Foo();\n    $f->bar();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_null_as_undefined_method() {
    // NullMethodCall fires, not UndefinedMethod
    let src = "<?php\nfunction test(): void {\n    $x = null;\n    $x->foo();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_defined_on_interface() {
    let src = "<?php\ninterface I {\n    public function doIt(): void;\n}\nfunction f(I $i): void {\n    $i->doIt();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_defined_on_abstract_class() {
    let src = "<?php\nabstract class Base {\n    abstract public function run(): void;\n}\nfunction f(Base $b): void {\n    $b->run();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_call_on_mixed() {
    // mixed type — method calls should not be flagged
    let src = "<?php\nfunction test(): void {\n    /** @var mixed $x */\n    $x = 1;\n    $x->anything();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn reports_missing_static_method() {
    // Static call to a method that does not exist on the class
    // line 4: "    Foo::missing();" — col 4 (Foo starts at col 4)
    let src = "<?php\nclass Foo {}\nfunction test(): void {\n    Foo::missing();\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedMethod", 4, 4);
}

#[test]
fn does_not_report_parent_method_that_exists() {
    let src = "<?php\nclass Base {\n    public function run(): void {}\n}\nclass Child extends Base {\n    public function doWork(): void {\n        parent::run();\n    }\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_generic_type_param() {
    // @template T on a function — method calls on the template param should not be flagged
    let src = "<?php\n/**\n * @template T\n * @param T $obj\n */\nfunction f($obj): void {\n    $obj->method();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_class_level_generic_return() {
    // @template T on a class — calling a method on the value returned by a generic getter
    // must not produce UndefinedMethod even when T is not concretely bound at call-site.
    let src = r#"<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    private mixed $value;

    /** @param T $value */
    public function __construct(mixed $value) {
        $this->value = $value;
    }

    /** @return T */
    public function get(): mixed {
        return $this->value;
    }
}

class User {
    public function getName(): string { return 'Alice'; }
}

$box = new Box(new User());
$user = $box->get();
$user->getName();
"#;
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_template_param_with_bound() {
    // @template T of object — bounded template param; method calls should still be allowed
    let src = "<?php\n/**\n * @template T of object\n * @param T $obj\n */\nfunction g($obj): void {\n    $obj->doSomething();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}
