===description===
Class alias already defined class
===ignore===
TODO
===file===
<?php
class A {}

class B {}

if (false) {
    class_alias(A::class, B::class);
}

function foo(A $a, B $b) : void {
    if ($a === $b) {}
}
===expect===
