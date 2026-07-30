===description===
D4: an arrow function with a declared return type also fires
MixedReturnStatement when returning a mixed value, mirroring the equivalent
`function(){...}` closure case (by-kind/mixed_return_statement/closure_fires.phpt).
===config===
suppress=UnusedVariable
===file===
<?php
$f = fn(): string => json_decode('{}');
===expect===
MixedReturnStatement@2:21-2:38: Cannot return a mixed type from function with declared return type 'string'
