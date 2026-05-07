===description===
reports override removes default
===file===
<?php
class Base {
    public function f(string $x = 'hi'): void { var_dump($x); }
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch@6:4: Method Child::f() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
