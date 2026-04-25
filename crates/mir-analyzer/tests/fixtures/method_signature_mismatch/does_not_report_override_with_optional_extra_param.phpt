===file===
<?php
class Base {
    public function f(): void {}
}
class Child extends Base {
    public function f(string $x = 'default'): void { var_dump($x); }
}
===expect===
