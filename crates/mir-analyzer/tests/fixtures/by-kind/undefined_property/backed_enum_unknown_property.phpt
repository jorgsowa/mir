===description===
backed enum unknown property
===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(Status $status): void {
    echo $status->label;
}
===expect===
UndefinedProperty@6:18-6:23: Property Status::$label does not exist
