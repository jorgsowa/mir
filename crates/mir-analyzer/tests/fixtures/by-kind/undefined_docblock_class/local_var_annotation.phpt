===description===
UndefinedDocblockClass fires when a local `@var` annotation names a class
that does not exist.
===config===
suppress=MixedAssignment
===file===
<?php
function process(): void {
    /** @var NonExistentVarClass $x */
    $x = fetchSomething();
    $x->doStuff();
}

function fetchSomething(): mixed {
    return null;
}
===expect===
UndefinedDocblockClass@4:4-4:26: Docblock type 'NonExistentVarClass' does not exist
