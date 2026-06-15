===description===
Deprecated class with new attr
===config===
suppress=UnusedVariable
===file===
<?php
#[\Deprecated]
class Foo { }

$a = new Foo();
===expect===
DeprecatedClass@5:9-5:12: Class Foo is deprecated
