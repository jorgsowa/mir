===description===
D4: each arrow function in a nested chain is checked against its own declared
return type independently — the inner mismatch must not be swallowed by the
outer arrow function's (correct) return type.
===config===
suppress=UnusedVariable
===file===
<?php
$f = fn(): string => (fn(): string => 123)();
===expect===
InvalidReturnType@2:38-2:41: Return type '123' is not compatible with declared 'string'
