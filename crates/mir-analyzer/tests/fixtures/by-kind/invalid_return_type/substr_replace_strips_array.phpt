===description===
FP-C: substr_replace returns string|array in stubs. When $string is a scalar
string, the return is always string. Strip the array case so callers don't
get InvalidReturnType or NullableReturnStatement.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function redact_middle(string $s): string {
    return substr_replace($s, '***', 2, -2);
}

function insert_at(string $s, string $ins, int $pos): string {
    return substr_replace($s, $ins, $pos, 0);
}
===expect===
