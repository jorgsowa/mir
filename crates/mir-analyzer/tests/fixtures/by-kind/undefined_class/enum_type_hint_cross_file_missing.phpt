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
Service.php: UndefinedClass@3:22-3:33: Class App\MissingEnum does not exist
Service.php: UndefinedClass@4:11-4:22: Class App\MissingEnum does not exist
