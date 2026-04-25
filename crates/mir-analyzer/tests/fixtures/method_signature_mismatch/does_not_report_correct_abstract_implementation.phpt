===file===
<?php
abstract class Base {
    abstract public function f(string $x): void;
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
}
===expect===
