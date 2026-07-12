===description===
A final class named only in a local `@var` docblock assertion must not be
reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Money {}

function process(): void {
    /** @var Money $m */
    $m = new stdClass();
    get_class($m);
}

process();
===expect===
