===description===
ImplicitToStringCast in echo statement
===file===
<?php
class Foo {}
$f = new Foo();
echo $f;
===expect===
ImplicitToStringCast@4:5-4:7: Class Foo is implicitly cast to string
