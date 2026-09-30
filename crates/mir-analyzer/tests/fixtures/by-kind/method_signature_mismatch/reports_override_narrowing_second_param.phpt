===description===
reports override narrowing second param
===config===
suppress=ForbiddenCode
===file===
<?php
class Base {
    public function f(string $x, string $y): void { var_dump($x, $y); }
}
class Child extends Base {
    public function f(string $x, int $y): void { var_dump($x, $y); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::f() signature mismatch: parameter $y type 'int' is incompatible with parent type 'string'
}
===expect===
