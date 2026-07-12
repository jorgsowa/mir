===description===
A final class named only in an `is_subclass_of($x, 'Foo')` string-literal
check must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {}

function check(object $x): bool {
    return is_subclass_of($x, 'Foo');
}

check(new stdClass());
===expect===
