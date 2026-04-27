===file:Service.php===
<?php
use App\MissingEnum;
function getStatus(): MissingEnum {
    return MissingEnum::Active;
}
===expect===
Service.php: UndefinedClass: Class App\MissingEnum does not exist
Service.php: UndefinedClass: Class App\MissingEnum does not exist
