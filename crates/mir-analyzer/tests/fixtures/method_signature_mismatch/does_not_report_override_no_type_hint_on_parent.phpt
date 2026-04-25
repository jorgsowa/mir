===file===
<?php
class Base {
    public function f($x): void { var_dump($x); }
}
class Child extends Base {
    public function f(int $x): void { var_dump($x); }
}
===expect===
