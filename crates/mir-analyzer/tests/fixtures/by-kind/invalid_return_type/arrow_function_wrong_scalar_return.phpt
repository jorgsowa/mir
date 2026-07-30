===description===
D4: arrow function body checked against its declared return type, same as a
regular closure/function body.
===config===
suppress=UnusedVariable
===file===
<?php
$f = fn(): int => 'not an int';
===expect===
InvalidReturnType@2:18-2:30: Return type '"not an int"' is not compatible with declared 'int'
