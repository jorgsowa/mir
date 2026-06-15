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
DeprecatedClass@7:9-7:12: Class Foo is deprecated
