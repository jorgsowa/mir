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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method C::foo() signature mismatch: return type 'B' is not a subtype of parent 'A'
        return new B();
    }
}
===expect===
