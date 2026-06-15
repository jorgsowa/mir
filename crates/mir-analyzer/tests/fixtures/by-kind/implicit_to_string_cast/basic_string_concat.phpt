===description===
ImplicitToStringCast in string concatenation
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$f = new Foo();
$s = 'Value: ' . $f;
===expect===
ImplicitToStringCast@4:17-4:19: Class Foo is implicitly cast to string
