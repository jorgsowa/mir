===source===
<?php
class Base {
    public function f(string $x, string $y): void { var_dump($x, $y); }
}
class Child extends Base {
    public function f(string $x, int $y): void { var_dump($x, $y); }
}
===expect===
MethodSignatureMismatch: Method Child::f() signature mismatch: parameter $y type 'int' is narrower than parent type 'string'
