===description===
A final class named only in a function's `@param` docblock tag (no native
param type naming it) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Bar {}

/**
 * @param ?Bar $b
 */
function useBar($b): void {
    if ($b) {
    }
}

useBar(null);
===expect===
