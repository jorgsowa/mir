===description===
Not visible clone method
===file===
<?php
class A {
    private function __clone() {}
}
$a = new A();
clone $a;
//<^^^^^^^^ InvalidClone: Call to private method A::__clone() from global scope
