===description===
instanceof cross file exists — no UndefinedClass when class is defined in another file
===file:Shape.php===
<?php
namespace App;
class Shape {}
===file:Checker.php===
<?php
use App\Shape;
function check(mixed $val): bool {
    return $val instanceof Shape;
}
===expect===
