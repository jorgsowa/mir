===description===
Trait return type mismatch — was previously undetected since trait-composed
methods weren't checked against the real parent (only own_methods() were).
===file===
<?php
class A {
    public function foo() : void {}
}

trait T {
    abstract public function foo() : string;
}

class B extends A {
    use T;
}
===expect===
MethodSignatureMismatch@7:4-7:44: Method B::foo() signature mismatch: return type 'string' is not a subtype of A::foo() 'void'
