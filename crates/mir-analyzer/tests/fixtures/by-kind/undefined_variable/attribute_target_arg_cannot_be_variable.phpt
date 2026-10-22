===description===
Attribute target arg cannot be variable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$target = 1;

#[Attribute($target)]
class Foo {}
===expect===
ParseError@4:12-4:19: Parse error: Constant expression contains invalid operations [invalid-constant-expression]
