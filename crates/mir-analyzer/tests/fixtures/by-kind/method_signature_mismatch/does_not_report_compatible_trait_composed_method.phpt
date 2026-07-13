===description===
A method composed purely via `use Trait;` with a signature compatible with
the real parent must not be flagged.
===config===
suppress=ForbiddenCode
===file===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
trait T {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    use T;
}
===expect===
