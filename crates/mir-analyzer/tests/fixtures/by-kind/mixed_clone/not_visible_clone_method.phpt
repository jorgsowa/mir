===description===
Not visible clone method
===file===
<?php
class A {
    private function __clone() {}
}
$a = new A();
clone $a;
===expect===
InvalidClone@6:0-6:8: cannot clone non-object A
