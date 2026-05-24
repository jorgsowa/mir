===description===
lessSpecificReturnStatement
===file===
<?php
class A {}
class B extends A {}

function foo(A $a): B {
    return $a;
}
===expect===
LessSpecificReturnStatement
===ignore===
TODO
