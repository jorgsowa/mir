===description===
reports override narrowing param type
===config===
suppress=ForbiddenCode
===file===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    public function f(int $x): void { var_dump($x); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::f() signature mismatch: parameter $x type 'int' is incompatible with parent type 'string'
}
===expect===
