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
}
===expect===
MethodSignatureMismatch@12:4-12:39: Method C::foo() signature mismatch: parameter $a type 'B' is narrower than parent type 'A'
