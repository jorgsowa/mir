===description===
Undefined mixin class with static method call
===file===
<?php
/** @mixin B */
//  ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
class A {}

A::foo();
//<^^^^^^^^ UndefinedMethod: Method A::foo() does not exist
===expect===
