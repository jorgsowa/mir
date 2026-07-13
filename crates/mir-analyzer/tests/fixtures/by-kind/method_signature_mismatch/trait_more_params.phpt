===description===
Trait more params — was previously undetected since trait-composed methods
weren't checked against the real parent (only own_methods() were).
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
MethodSignatureMismatch@7:4-7:53: Method B::foo() signature mismatch: return type 'string' is not a subtype of A::foo() 'void'
MethodSignatureMismatch@7:4-7:53: Method B::foo() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
