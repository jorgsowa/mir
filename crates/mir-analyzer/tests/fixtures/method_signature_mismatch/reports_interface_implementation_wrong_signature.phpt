===file===
<?php
interface I {
    public function f(string $x): void;
}
class C implements I {
    public function f(int $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch: Method C::f() signature mismatch: parameter $x type 'int' is narrower than parent type 'string'
