===description===
is_invalid_for_access omitted TIntegralFloat (floor()/ceil()'s return type),
unlike the definite-invalid check right above it, so an array|TIntegralFloat
union silently skipped PossiblyInvalidArrayAccess.
===config===
suppress=UnusedVariable
===file===
<?php
function test(array $arr, float $n, bool $cond): void {
    $x = $cond ? $arr : floor($n);
    $x[0];
}
===expect===
PossiblyInvalidArrayAccess@4:4-4:9: Possibly invalid array access: 'array|float' might not support []
