===description===
`SELF` in a catch clause is the `self` keyword, not a class name: only the
compile error is reported, never UndefinedClass.
===file===
<?php
class Foo {
    public function f(): void {
        try {
        } catch (SELF $e) {
//               ^^^^ CompileError: Bad class name in the catch statement [class-keyword-in-catch]
        }
    }
}
