===description===
A class_exists() guard on the receiver class suppresses TooManyArguments
for a static call inside the guarded branch: the live signature may have
more parameters than the snapshot, so arity is untrustworthy.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
class NewApi {
    public static function run(int $a): void {}
}

function check(): void {
    if (class_exists('NewApi')) {
        NewApi::run(1, 2);
    }
}
===expect===
