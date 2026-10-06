===description===
Negative control for the abstract-base arity fix: TooFewArguments is NOT
suppressed through an abstract-typed receiver — a concrete override can
never REQUIRE more params than its abstract declaration, so the abstract
method's own required-param count is still a reliable lower bound.
===file===
<?php
abstract class Base {
    abstract public function configure(int $a, int $b): void;
}
class Foo extends Base {
    public function configure(int $a, int $b, ?object $svc = null): void {}
//                                            ^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $svc is never used
}
class T {
    private Base $foo;
    public function __construct() {
        $this->foo = new Foo();
    }
    public function run(): void {
        $this->foo->configure(1);
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for configure(): expected 2, got 1
    }
}
