===description===
Undefined mixin class with property assignment
===ignore===
TODO
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo = "bar";
===expect===
