===description===
A built class name typed `non-empty-string` is a valid `new` or `::` receiver.
===file===
<?php
function f(string $class): object {
    $queryClass = $class . 'Query';
    $queryClass::create();
    return new $queryClass();
}
