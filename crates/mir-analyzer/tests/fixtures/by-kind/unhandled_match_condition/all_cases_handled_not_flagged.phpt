===description===
UnhandledMatchCondition does NOT fire when all enum cases are covered.
===file===
<?php
enum Status {
    case Active;
    case Inactive;
}

function label(Status $s): string {
    return match ($s) {
        Status::Active => 'active',
        Status::Inactive => 'inactive',
    };
}
===expect===
