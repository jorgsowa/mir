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
//  ^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentVarClass' does not exist
    $x->doStuff();
}

function fetchSomething(): mixed {
    return null;
}
===expect===
