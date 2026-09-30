===description===
backed enum unknown property
===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(Status $status): void {
    echo $status->label;
//                ^^^^^ UndefinedProperty: Property Status::$label does not exist
}
===expect===
