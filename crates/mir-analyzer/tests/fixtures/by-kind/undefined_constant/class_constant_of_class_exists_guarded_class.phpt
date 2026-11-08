===description===
A constant of a class proven only by `class_exists()` is unknown, not undefined.
===file===
<?php
namespace App;

use Gen\Installed;

function a(): void {
    echo class_exists(Installed::class) ? Installed::HASH : 'v';
}
function b(): void {
    if (class_exists(Installed::class)) {
        echo Installed::HASH;
    }
}
