===description===
Redefined trait method in subclass
===file===
<?php
trait T {
    public function fooFoo(): void {
    }
}

class B {
    use T;
}

class C extends B {
    public function fooFoo(string $a): void {
    }
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
