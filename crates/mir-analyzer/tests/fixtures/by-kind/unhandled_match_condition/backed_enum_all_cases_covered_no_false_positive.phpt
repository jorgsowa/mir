===description===
UnhandledMatchCondition does not fire for a backed enum match that covers every case.
===file===
<?php
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}

function label(Status $s): string {
    return match($s) {
        Status::Active => "active",
        Status::Inactive => "inactive",
    };
}
===expect===
