===description===
Template params used as bounds for other template params must not be FQN-qualified.
Tests method-level (W of K), class-level (V of K), and function-level (B of A) cases.
===config===
suppress=UnusedVariable
===file:Util/Binder.php===
<?php
namespace Util;

class Base {}
class Child extends Base {}

/**
 * @template A
 * @template B of A
 * @param A $a
 * @param B $b
 * @return B
 */
function bind(mixed $a, mixed $b): mixed {
    return $a ?? $b;
}

/**
 * @template K
 * @template V of K
 */
class Pair {
    /**
     * @template W of K
     * @param W $w
     * @return W
     */
    public function map(mixed $w): mixed {
        return $w;
    }
}
===file:App/UseIt.php===
<?php
namespace App;

use Util\Base;
use Util\Child;
use Util\Pair;

$b = new Base();
$c = new Child();

// Function: template param (A) as bound for B — no spurious violation
$result = \Util\bind($b, $c);

// Method: template param (K) as bound for W — no spurious violation
$pair = new Pair();
$mapped = $pair->map($b);
===expect===
