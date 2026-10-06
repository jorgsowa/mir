===description===
ImplicitToStringCast in heredoc with interpolation
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
$f = new Foo();
$s = <<<EOT
Value: {$f}
//      ^^ ImplicitToStringCast: Class Foo is implicitly cast to string
EOT;
