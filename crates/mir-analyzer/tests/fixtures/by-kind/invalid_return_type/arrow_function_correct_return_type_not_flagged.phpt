===description===
D4 negative control: an arrow function whose inferred return type actually
satisfies its declared return type must not be flagged.
===config===
suppress=UnusedVariable
===file===
<?php
$f = fn(): int => 123;
$g = fn(): string => (string) 123;
===expect===
