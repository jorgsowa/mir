===description===
Non nullable subclass param
===file===
<?php
class A {
    public function foo(?string $s): string {
        return $s !== null ? $s : "hello";
    }
}

class B extends A {
    public function foo(string $s): string {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foo() signature mismatch: parameter $s type 'string' is narrower than parent type 'string|null'
        return $s;
    }
}
