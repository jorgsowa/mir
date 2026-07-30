===description===
D4: a `static fn` is checked the same as a non-static arrow function — the
return-type check must not be skipped just because there's no captured $this.
===config===
suppress=UnusedVariable
===file===
<?php
$f = static fn(): int => 'not an int';
===expect===
InvalidReturnType@2:25-2:37: Return type '"not an int"' is not compatible with declared 'int'
