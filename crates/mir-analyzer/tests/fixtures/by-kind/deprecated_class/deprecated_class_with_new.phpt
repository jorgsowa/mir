===description===
Deprecated class with new
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @deprecated
 */
class Foo { }

$a = new Foo();
//       ^^^ DeprecatedClass: Class Foo is deprecated
===expect===
