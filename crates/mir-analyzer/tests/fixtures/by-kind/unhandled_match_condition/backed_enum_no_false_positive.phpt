===description===
UnhandledMatchCondition does NOT fire for backed enums — their case values are not
exhaustiveness-checked (the analyzer cannot enumerate all possible backing values).
===file===
<?php
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
    case Pending = 'pending';
}

function label(Status $s): string {
    return match($s) {
        Status::Active => "active",
    };
}
===expect===
