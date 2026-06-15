===description===
ImplicitToStringCast in heredoc with interpolation
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$f = new Foo();
$s = <<<EOT
Value: {$f}
EOT;
===expect===
ImplicitToStringCast@5:8-5:10: Class Foo is implicitly cast to string
