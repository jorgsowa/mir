===description===
ImplicitToStringCast in string interpolation
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$f = new Foo();
$s = "Value: {$f}";
===expect===
ImplicitToStringCast@4:15-4:17: Class Foo is implicitly cast to string
