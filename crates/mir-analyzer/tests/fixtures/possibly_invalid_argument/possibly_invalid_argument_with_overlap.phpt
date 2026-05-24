===description===
possiblyInvalidArgumentWithOverlap
===file===
<?php
class A {}
class B {}
class C {}

$foo = rand(0, 1) ? new A : new B;

/** @param B|C $b */
function bar($b) : void {}

bar($foo);
===expect===
PossiblyInvalidArgument
===ignore===
TODO
