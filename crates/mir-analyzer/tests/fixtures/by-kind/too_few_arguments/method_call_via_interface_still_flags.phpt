===description===
Same negative control as the abstract-base one, through an interface
receiver: TooFewArguments still fires since an implementation can never
require more params than the interface declares.
===file===
<?php
interface Base {
    public function configure(int $a, int $b): void;
}
class Foo implements Base {
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
===expect===
