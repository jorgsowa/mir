===description===
ImplicitToStringCast in string concatenation
===file===
<?php
class Foo {}
$f = new Foo();
$s = 'Value: ' . $f;
===expect===
ImplicitToStringCast@4:17: Class Foo does not implement __toString()
