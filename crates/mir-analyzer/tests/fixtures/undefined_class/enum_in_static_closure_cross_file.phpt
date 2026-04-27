===file:Status.php===
<?php
namespace App;
enum Status {
    case Active;
    case Inactive;
}
===file:Service.php===
<?php
namespace App\Service;
use App\Status;
function getCallback(): \Closure {
    return static fn() => Status::Active;
}
===expect===
