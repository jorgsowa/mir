===description===
Negative control for the array_multisort leniency fix: sort() (and other
ordinary by-ref array functions) still require an lvalue — the leniency is
specific to array_multisort's own special-cased engine behavior, not a
blanket relaxation of by-ref checking.
===file===
<?php
function make(): array {
    return [3, 1, 2];
}
function test(): void {
    sort(make());
}
===expect===
InvalidPassByReference@6:9-6:15: Argument $array of sort() must be passed by reference
