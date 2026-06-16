===description===
(float) cast on a union that includes array but also scalar-safe atoms does not emit InvalidCast
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function option(string $key): string|array|bool|null {
    return null;
}

$value = (float) option('rate');
===expect===
