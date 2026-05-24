===description===
deprecatedClassWithNewAttr
===file===
<?php
#[\Deprecated]
class Foo { }

$a = new Foo();
===expect===
DeprecatedClass
===ignore===
TODO
