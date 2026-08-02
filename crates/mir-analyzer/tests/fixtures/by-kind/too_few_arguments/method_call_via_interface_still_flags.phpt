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
}
class T {
    private Base $foo;
    public function __construct() {
        $this->foo = new Foo();
    }
    public function run(): void {
        $this->foo->configure(1);
    }
}
===expect===
UnusedParam@6:46-6:65: Parameter $svc is never used
TooFewArguments@14:8-14:32: Too few arguments for configure(): expected 2, got 1
