===description===
Unlike the int/float siblings, (string) cast still emits InvalidCast when the
union includes array, even alongside scalar-safe atoms — PHP's "Array to
string conversion" warning fires on the array branch regardless of what
else the union contains.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function option(string $key): string|array|bool|null {
    return null;
}

$timeout = (string) option('timeout');
===expect===
InvalidCast@6:20-6:37: Cannot cast 'string|array|bool|null' to 'string'
