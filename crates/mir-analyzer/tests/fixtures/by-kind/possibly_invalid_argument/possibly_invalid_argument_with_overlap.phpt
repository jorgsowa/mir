===description===
Possibly invalid argument with overlap
===config===
suppress=UnusedParam
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
PossiblyInvalidArgument@11:5-11:9: Argument $b of bar() expects 'B|C', possibly different type 'A|B' provided
