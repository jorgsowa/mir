===description===
UnhandledMatchCondition fires for a nullable enum subject when neither a `null` arm nor `default` is present, even if every non-null case is covered.
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
    };
}
===expect===
UnhandledMatchCondition@8:11-11:5: Unhandled match condition: null
