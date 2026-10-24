===description===
An attribute argument must be a constant expression, not a variable.
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
//          ^^^^^^^ CompileError: Compile error: Constant expression contains invalid operations [invalid-constant-expression]
class Foo {}
