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
        Status::Active => "active",
    };
}
===expect===
UnhandledMatchCondition@9:11-11:5: Unhandled match condition: Status::Inactive, Status::Pending
