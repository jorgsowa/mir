===description===
undefinedMixinClassWithPropertyFetch
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo;
===expect===
UndefinedPropertyFetch
===ignore===
TODO
