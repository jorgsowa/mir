===description===
Negative control for the abstract/interface arity fix: a receiver typed as
a plain, concrete (non-abstract) base class still gets its own arity
checked normally — the fix only suppresses TooManyArguments when the
resolved method has no concrete body of its own.
===file===
<?php
class Base {
    public function configure(): void {}
}
class T {
    private Base $foo;
    public function __construct() {
        $this->foo = new Base();
    }
    public function run(): void {
        $this->foo->configure(new stdClass());
    }
}
===expect===
TooManyArguments@11:30-11:44: Too many arguments for configure(): expected 0, got 1
