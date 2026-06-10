===description===
Less specific return statement
===ignore===
TODO
===file===
<?php
class A {}
class B extends A {}

function foo(A $a): B {
    return $a;
}
===expect===
