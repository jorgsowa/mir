===description===
nullable backed enum value without nullsafe
===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(?Status $status): string {
    return $status->value;
}
===expect===
PossiblyNullPropertyFetch@6:11: Cannot access property $value on possibly null value
===ignore===
TODO
