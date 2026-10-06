===description===
UnhandledMatchCondition fires for backed enums too — a backed enum's case
set is finite and enumerable regardless of its backing scalar type, so
missing cases are still reported (real PHP throws UnhandledMatchError here).
===file===
<?php
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
    case Pending = 'pending';
}

function label(Status $s): string {
    return match($s) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: Status::Inactive, Status::Pending
        Status::Active => "active",
    };
}
