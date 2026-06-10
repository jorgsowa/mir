===description===
Trait more params
===ignore===
TODO
===file===
<?php
class A {
    public function foo() : void {}
}

trait T {
    abstract public function foo(string $s) : string;
}

class B extends A {
    use T;
}
===expect===
