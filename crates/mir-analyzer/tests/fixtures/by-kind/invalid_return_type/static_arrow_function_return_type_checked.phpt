===description===
D4: a `static fn` is checked the same as a non-static arrow function — the
return-type check must not be skipped just because there's no captured $this.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$f = static fn(): int => 'not an int';
//                       ^^^^^^^^^^^^ InvalidReturnType: Return type '"not an int"' is not compatible with declared 'int'
===expect===
