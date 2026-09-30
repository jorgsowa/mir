===description===
ImplicitToStringCast in echo statement
===file===
<?php
class Foo {}
$f = new Foo();
echo $f;
//   ^^ ImplicitToStringCast: Class Foo is implicitly cast to string
===expect===
