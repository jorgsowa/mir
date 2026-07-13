===description===
Sibling of int/float_cast_mixed_union_with_scalars_no_error: (string) cast
on a union that includes array but also scalar-safe atoms does not emit
InvalidCast — the scalar atoms make the cast valid.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function option(string $key): string|array|bool|null {
    return null;
}

$timeout = (string) option('timeout');
===expect===
