===file:Status.php===
<?php
namespace App;
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}
===file:Service.php===
<?php
use App\Status;
function getAll(): array {
    return Status::cases();
}
===expect===
