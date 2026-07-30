===description===
Same underlying gap as the abstract-class repro, but through an interface
receiver — an interface method has no body at all (implicitly a contract
only), so an implementing class is equally free to add extra optional
params beyond the interface's own declared signature.
===file===
<?php
interface Base {
    public function configure(): void;
}
class Foo implements Base {
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
