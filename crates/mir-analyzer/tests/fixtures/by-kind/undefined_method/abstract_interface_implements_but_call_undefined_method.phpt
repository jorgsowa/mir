===description===
Abstract interface implements but call undefined method
===file===
<?php
interface I {
    public function foo() : void;
}

abstract class A implements I {
    public function bar(): void {
        $this->foo2();
    }
}
===expect===
UndefinedMethod
===ignore===
TODO
