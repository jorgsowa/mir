===description===
enum type hint cross file missing
===config===
suppress=MixedReturnStatement
===file:Service.php===
<?php
use App\MissingEnum;
function getStatus(): MissingEnum {
    return MissingEnum::Active;
}
===expect===
Service.php: UndefinedClass@3:23-3:34: Class App\MissingEnum does not exist
Service.php: UndefinedClass@4:12-4:23: Class App\MissingEnum does not exist
