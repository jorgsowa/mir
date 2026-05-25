===description===
Undefined mixin class with property assignment
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo = "bar";
===expect===
UndefinedPropertyAssignment
===ignore===
TODO
