===description===
A bare `iterable` @return/@param docblock type expands internally to
`array|Traversable`. The `Traversable` half must resolve as PHP's global
built-in interface, not get namespace-qualified against the file's own
namespace (which would misreport it as an undefined class).
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @param iterable $value
 * @return iterable
 */
function passthrough($value): iterable {
    return $value;
}
===expect===
