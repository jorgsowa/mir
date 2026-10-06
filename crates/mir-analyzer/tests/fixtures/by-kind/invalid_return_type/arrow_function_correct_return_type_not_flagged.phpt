===description===
D4 negative control: an arrow function whose inferred return type actually
satisfies its declared return type must not be flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$f = fn(): int => 123;
$g = fn(): string => (string) 123;
