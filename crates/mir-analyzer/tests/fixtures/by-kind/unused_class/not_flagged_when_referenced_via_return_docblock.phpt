===description===
A final class named only in a function's `@return` docblock tag (no native
return type naming it) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {}

/**
 * @return ?Foo
 */
function makeFoo(): mixed {
    return null;
}

makeFoo();
===expect===
