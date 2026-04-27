===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(Status $status): void {
    echo $status->label;
}
===expect===
UndefinedProperty: Property Status::$label does not exist
