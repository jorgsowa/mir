===description===
enum type hint cross file missing
===config===
suppress=MixedReturnStatement
===file:Service.php===
<?php
use App\MissingEnum;
function getStatus(): MissingEnum {
//                    ^^^^^^^^^^^ UndefinedClass: Class App\MissingEnum does not exist
    return MissingEnum::Active;
//         ^^^^^^^^^^^ UndefinedClass: Class App\MissingEnum does not exist
}
===expect===
