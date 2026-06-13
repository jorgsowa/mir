===description===
Redefined trait method in subclass
===config===
suppress=UnusedParam
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
MethodSignatureMismatch@12:4-12:45: Method C::foofoo() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
