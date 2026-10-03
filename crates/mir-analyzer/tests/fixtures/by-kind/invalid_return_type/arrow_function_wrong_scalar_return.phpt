===description===
D4: arrow function body checked against its declared return type, same as a
regular closure/function body.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$f = fn(): int => 'not an int';
//                ^^^^^^^^^^^^ InvalidReturnType: Return type '"not an int"' is not compatible with declared 'int'
===expect===
