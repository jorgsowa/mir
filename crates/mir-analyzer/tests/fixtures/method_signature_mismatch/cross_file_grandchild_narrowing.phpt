===file:GrandParent.php===
<?php
class GrandParent {
    public function f(string $x): void { var_dump($x); }
}
===file:Parent.php===
<?php
class ParentClass extends GrandParent {}
===file:GrandChild.php===
<?php
class GrandChild extends ParentClass {
    public function f(int $x): void { var_dump($x); }
}
===expect===
GrandChild.php: MethodSignatureMismatch: Method GrandChild::f() signature mismatch: parameter $x type 'int' is narrower than parent type 'string'
