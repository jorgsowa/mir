===description===
ImplicitToStringCast in echo statement
===file===
<?php
class Foo {}
$f = new Foo();
echo $f;
===expect===
ImplicitToStringCast@4:6-4:8: Class Foo does not implement __toString()
