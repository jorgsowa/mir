===description===
Wrong case class name in new expression is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
$x = new foo();
//       ^^^ WrongCaseClass: Class name 'foo' has incorrect casing; use 'Foo'
