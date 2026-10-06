===description===
D4: an arrow function with a declared return type also fires
MixedReturnStatement when returning a mixed value, mirroring the equivalent
`function(){...}` closure case (by-kind/mixed_return_statement/closure_fires.phpt).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$f = fn(): string => json_decode('{}');
//                   ^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
