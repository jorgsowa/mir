===description===
nullable backed enum value without nullsafe
===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(?Status $status): string {
    return $status->value;
//  ^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
//         ^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $value on possibly null value
}
