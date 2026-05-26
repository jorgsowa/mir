===description===
Undefined mixin class with property fetch
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo;
===expect===
UndefinedPropertyFetch
===ignore===
TODO
