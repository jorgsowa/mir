===description===
A dynamic-name compact() call (variadic or array argument) exempts variables in scope from UnusedVariable, since the actual names read are unknowable statically.
===file===
<?php
function view(array $names): array {
    $title = 'Hello';
    $body = 'World';
    return compact(...$names);
}
===expect===
