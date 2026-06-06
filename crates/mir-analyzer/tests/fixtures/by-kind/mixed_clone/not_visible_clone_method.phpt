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
InvalidClone@6:1-6:9: cannot clone non-object A
