===description===
A class_exists() guard on a resolvable class does NOT suppress the
constructor-arity check for new in the guarded branch: the guard is true
in the snapshot's own environment, so the branch is live and the
no-arg snapshot constructor is authoritative.
===config===
suppress=MissingReturnType
===file===
<?php
class NewApi {}

function create(): void {
    if (class_exists('NewApi')) {
        new NewApi(1);
    }
}
===expect===
TooManyArguments@6:8-6:21: Too many arguments for NewApi::__construct(): expected 0, got 1
