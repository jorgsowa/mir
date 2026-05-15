===description===
function parameter type hint cross file exists — no UndefinedClass when class is defined in another file
===file:Service.php===
<?php
namespace App;
class Service {}
===file:Consumer.php===
<?php
use App\Service;
function process(Service $svc): void {
    echo get_class($svc);
}
===expect===
