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
===expect===
DeprecatedClass@7:10-7:13: Class Foo is deprecated
