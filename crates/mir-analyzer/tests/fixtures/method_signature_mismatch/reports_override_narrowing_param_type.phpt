===description===
reports override narrowing param type
===file===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    public function f(int $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch@6:4: Method Child::f() signature mismatch: parameter $x type 'int' is narrower than parent type 'string'
===ignore===
TODO
