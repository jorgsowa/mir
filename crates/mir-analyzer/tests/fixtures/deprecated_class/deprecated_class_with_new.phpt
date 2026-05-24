===description===
deprecatedClassWithNew
===file===
<?php
/**
 * @deprecated
 */
class Foo { }

$a = new Foo();
===expect===
DeprecatedClass
===ignore===
TODO
