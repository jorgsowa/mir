===description===
ImplicitToStringCast in string interpolation
===file===
<?php
class Foo {}
$f = new Foo();
$s = "Value: {$f}";
===expect===
ImplicitToStringCast@4:15: Class Foo does not implement __toString()
