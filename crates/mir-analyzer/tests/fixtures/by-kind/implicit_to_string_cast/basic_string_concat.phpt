===description===
ImplicitToStringCast in string concatenation
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
$s = 'Value: ' . $f;
//               ^^ ImplicitToStringCast: Class Foo is implicitly cast to string
===expect===
