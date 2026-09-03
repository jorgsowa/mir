===description===
Calling a method through a receiver typed as an abstract class must not
flag TooManyArguments against the abstract declaration's own (minimal)
param count — a concrete override is always free to add extra OPTIONAL
params beyond what the abstract method declares (valid PHP override
variance), so the abstract signature's total param count can't be trusted
as an upper bound on any concrete call.
===file===
<?php
abstract class Base {
    abstract public function configure(): void;
}
class Foo extends Base {
    public function configure(?object $svc = null): void {}
}
class T {
    private Base $foo;
    public function __construct() {
        $this->foo = new Foo();
    }
    public function run(): void {
        $this->foo->configure(new stdClass());
    }
}
===expect===
UnusedParam@6:30-6:49: Parameter $svc is never used
