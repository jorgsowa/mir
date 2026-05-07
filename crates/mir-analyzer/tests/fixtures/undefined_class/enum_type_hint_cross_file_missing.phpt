===description===
enum type hint cross file missing
===file:Service.php===
<?php
use App\MissingEnum;
function getStatus(): MissingEnum {
    return MissingEnum::Active;
}
===expect===
Service.php: UndefinedClass@3:22: Class App\MissingEnum does not exist
Service.php: UndefinedClass@4:11: Class App\MissingEnum does not exist
