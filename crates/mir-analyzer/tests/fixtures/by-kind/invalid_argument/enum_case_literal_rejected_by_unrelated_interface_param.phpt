===description===
Negative control for the enum-case/interface subtype fix: an enum-case
literal must still be rejected when passed to a parameter typed as an
interface its declaring enum does NOT implement.
===config===
php_version=8.1
suppress=UnusedParam
===file===
<?php
interface Unrelated {
    public function nope(): void;
}

enum Status {
    case Active;
    case Inactive;
}

function needsUnrelated(Unrelated $x): void {}

function test(Status $s): void {
    if ($s === Status::Active) {
        needsUnrelated($s);
    }
}
===expect===
InvalidArgument@15:23-15:25: Argument $x of needsUnrelated() expects 'Unrelated', got 'Status::Active'
