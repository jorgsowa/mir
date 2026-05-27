===description===
Intersection bound where parts are referenced via use-import alias — aliases should be resolved in bound
===file:contracts.php===
<?php
namespace App\Contracts;

interface Countable {}
interface Stringable {}
class Both implements Countable, Stringable {}
===file:user.php===
<?php
namespace App\Sub;

use App\Contracts\Countable;
use App\Contracts\Stringable;
use App\Contracts\Both;

/**
 * @template T of Countable&Stringable
 * @param T $val
 */
function f($val): void {
    $val;
}

f(new Both());
===expect===
