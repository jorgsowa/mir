===description===
UnhandledMatchCondition does not fire for a nullable enum subject when a `null` arm covers the null case alongside every enum case.
===file===
<?php
enum Status {
    case Active;
    case Inactive;
}

function label(?Status $s): string {
    return match($s) {
        Status::Active => "active",
        Status::Inactive => "inactive",
        null => "none",
    };
}
===expect===
