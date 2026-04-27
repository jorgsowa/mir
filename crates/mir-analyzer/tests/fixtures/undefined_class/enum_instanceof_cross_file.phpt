===file:Status.php===
<?php
namespace App;
enum Status: string {
    case Active = 'active';
}
===file:Checker.php===
<?php
use App\Status;
function check(mixed $val): bool {
    return $val instanceof Status;
}
===expect===
