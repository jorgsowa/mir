===description===
(int) cast on a union that includes array but also scalar-safe atoms (string|bool|null)
does not emit InvalidCast — the scalar atoms make the cast valid
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function option(string $key): string|array|bool|null {
    return null;
}

$timeout = (int) option('timeout');
$retries = (int) option('retries');
===expect===
