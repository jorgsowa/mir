===description===
ImplicitToStringCast in print statement
===file===
<?php
class Foo {}
$f = new Foo();
print $f;
===expect===
ImplicitToStringCast@4:7: Class Foo does not implement __toString()
