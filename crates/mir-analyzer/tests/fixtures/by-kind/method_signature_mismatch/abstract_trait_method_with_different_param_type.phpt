===description===
Abstract trait method with different param type. The implementing method's native
parameter type (B) is incompatible with the trait's abstract requirement (A), an LSP
violation PHP rejects — mirrors the return-type sibling fixture. (G4)
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B {}

trait T {
    abstract public function foo(A $a) : void;
}

class C {
    use T;

    public function foo(B $a) : void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method C::foo() signature mismatch: parameter $a type 'B' is incompatible with parent type 'A'
}
===expect===
