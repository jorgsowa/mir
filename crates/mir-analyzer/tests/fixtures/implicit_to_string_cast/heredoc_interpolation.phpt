===description===
ImplicitToStringCast in heredoc with interpolation
===file===
<?php
class Foo {}
$f = new Foo();
$s = <<<EOT
Value: {$f}
EOT;
===expect===
ImplicitToStringCast@5:8: Class Foo does not implement __toString()
