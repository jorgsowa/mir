===source===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    public function f(int $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch: Method Child::f() signature mismatch: parameter $x type 'int' is narrower than parent type 'string'
