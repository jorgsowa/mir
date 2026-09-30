===description===
ImplicitToStringCast in print statement
===file===
<?php
class Foo {}
$f = new Foo();
print $f;
//    ^^ ImplicitToStringCast: Class Foo is implicitly cast to string
===expect===
