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
NullableReturnStatement@6:4-6:26: Return type 'string|null' is not compatible with declared 'string'
PossiblyNullPropertyFetch@6:11-6:25: Cannot access property $value on possibly null value
