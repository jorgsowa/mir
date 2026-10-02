===description===
Undefined mixin class with static method call
===file===
<?php
/** @mixin B */
class A {}

A::foo();
//<^^^^^^^^ UndefinedMethod: Method A::foo() does not exist
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
