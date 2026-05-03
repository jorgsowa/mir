===description===
backed enum string value correct return
===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(Status $status): string {
    return $status->value;
}
===expect===
===ignore===
TODO
