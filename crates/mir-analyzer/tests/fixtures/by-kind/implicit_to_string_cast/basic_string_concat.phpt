===description===
ImplicitToStringCast in string concatenation
===file===
<?php
class Foo {}
$f = new Foo();
$s = 'Value: ' . $f;
===expect===
ImplicitToStringCast@4:18-4:20: Class Foo is implicitly cast to string
