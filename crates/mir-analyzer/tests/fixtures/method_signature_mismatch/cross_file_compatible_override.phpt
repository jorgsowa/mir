===file:Base.php===
<?php
class Base {
    public function process(string $x): void { var_dump($x); }
}
===file:Child.php===
<?php
class Child extends Base {
    public function process(string $x): void { var_dump($x); }
}
===expect===
