===description===
A class_exists() guard on the constructed class suppresses the
constructor-arity diagnostic for a `new` inside the guarded branch: the
live constructor may accept different arguments than the snapshot.
===config===
suppress=MissingReturnType
===file===
<?php
class NewApi {}

function check(): void {
    if (class_exists('NewApi')) {
        new NewApi(1);
    }
}
===expect===
