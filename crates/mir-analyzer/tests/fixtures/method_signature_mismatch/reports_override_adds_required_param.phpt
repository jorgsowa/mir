===source===
<?php
class Base {
    public function f(): void {}
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
}
===expect===
MethodSignatureMismatch at 1:0
