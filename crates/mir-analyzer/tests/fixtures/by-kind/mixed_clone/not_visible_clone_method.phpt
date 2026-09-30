===description===
Not visible clone method
===file===
<?php
class A {
    private function __clone() {}
}
$a = new A();
clone $a;
//<^^^^^^^^ InvalidClone: cannot clone non-object A
===expect===
