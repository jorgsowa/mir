===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(Status $status): string {
    return $status->value;
}
===expect===
