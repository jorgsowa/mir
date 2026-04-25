===file===
<?php
class Base {
    public function f(): void {}
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch: Method Child::f() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
