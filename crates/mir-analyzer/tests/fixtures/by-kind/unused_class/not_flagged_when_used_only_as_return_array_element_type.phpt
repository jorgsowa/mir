===description===
A final class named only as the element type of an `array<int, Foo>` @return docblock shape must not be reported unused.
===config===
suppress=
===file===
<?php
final class Item {}

/** @return array<int, Item> */
function all(): array {
    return [];
}

all();
===expect===
