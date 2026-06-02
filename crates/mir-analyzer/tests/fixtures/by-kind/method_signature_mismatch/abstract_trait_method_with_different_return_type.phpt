===description===
Abstract trait method with different return type
===file===
<?php
class A {}
class B {}

trait T {
    abstract public function foo() : A;
}

class C {
    use T;

    public function foo() : B{
        return new B();
    }
}
===expect===
MethodSignatureMismatch@12:4-12:30: Method C::foo() signature mismatch: return type 'B' is not a subtype of parent 'A'
