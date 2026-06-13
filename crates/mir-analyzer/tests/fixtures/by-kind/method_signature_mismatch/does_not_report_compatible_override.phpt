===description===
does not report compatible override
===config===
suppress=ForbiddenCode
===file===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
}
===expect===
