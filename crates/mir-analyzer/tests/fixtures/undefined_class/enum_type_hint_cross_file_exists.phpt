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
function getStatus(): Status {
    return Status::Active;
}
function checkStatus(Status $s): string { return $s->value; }
===expect===
