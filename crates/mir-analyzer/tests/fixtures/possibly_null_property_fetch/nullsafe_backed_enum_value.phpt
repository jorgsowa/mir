===file===
<?php
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}
function test(?Status $status): string {
    return $status?->value ?? 'unknown';
}
===expect===
