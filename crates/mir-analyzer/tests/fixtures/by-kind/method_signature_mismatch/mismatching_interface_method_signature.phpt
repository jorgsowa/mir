===description===
Mismatching interface method signature
===config===
suppress=UnusedParam
===file===
<?php
interface A {
    public function fooFoo(int $a): void;
}

class B implements A {
    public function fooFoo(string $a): void {

    }
}
===expect===
MethodSignatureMismatch@7:4-7:45: Method B::foofoo() signature mismatch: parameter $a type 'string' is narrower than parent type 'int'
