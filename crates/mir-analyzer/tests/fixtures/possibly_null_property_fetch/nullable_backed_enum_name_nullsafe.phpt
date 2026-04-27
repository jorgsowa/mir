===file===
<?php
enum Status: string {
    case Active = 'active';
}
function test(?Status $status): string {
    return $status?->name ?? 'unknown';
}
===expect===
