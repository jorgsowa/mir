===description===
ImplicitToStringCast in string concatenation
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$f = new Foo();
$s = 'Value: ' . $f;
//               ^^ ImplicitToStringCast: Class Foo is implicitly cast to string
===expect===
