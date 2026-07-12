===description===
A final class named only in an `is_a($x, 'Foo')` string-literal check must
not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {}

function check(object $x): bool {
    return is_a($x, 'Foo');
}

check(new stdClass());
===expect===
