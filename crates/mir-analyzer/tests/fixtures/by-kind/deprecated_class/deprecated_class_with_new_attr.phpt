===description===
Deprecated class with new attr
===config===
suppress=UnusedVariable
===file===
<?php
#[\Deprecated]
class Foo { }

$a = new Foo();
//       ^^^ DeprecatedClass: Class Foo is deprecated
===expect===
