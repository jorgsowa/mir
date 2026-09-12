===description===
A class_exists() guard on the receiver class suppresses TooFewArguments
for a static call inside the guarded branch: the live signature may have
more required parameters than the snapshot.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
class NewApi {
    public static function run(int $a, int $b): void {}
}

function check(): void {
    if (class_exists('NewApi')) {
        NewApi::run(1);
    }
}
===expect===
