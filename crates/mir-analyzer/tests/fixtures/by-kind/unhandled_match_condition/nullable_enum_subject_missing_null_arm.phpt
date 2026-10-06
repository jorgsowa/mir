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
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: null
        Status::Active => "active",
        Status::Inactive => "inactive",
    };
}
