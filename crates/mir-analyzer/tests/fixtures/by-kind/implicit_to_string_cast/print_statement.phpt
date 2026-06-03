===description===
ImplicitToStringCast in print statement
===file===
<?php
class Foo {}
$f = new Foo();
print $f;
===expect===
ImplicitToStringCast@4:7-4:9: Class Foo is implicitly cast to string
