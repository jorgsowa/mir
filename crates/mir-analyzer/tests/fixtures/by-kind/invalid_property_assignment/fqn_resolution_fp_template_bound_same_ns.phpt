===description===
Template bound referencing a same-namespace class is FQN-qualified and no longer produces a spurious bound violation
===config===
suppress=UnusedVariable
===file:Lib/Container.php===
<?php
namespace Lib;

class Base {}
class Child extends Base {}

/**
 * @template T of Base
 * @param T $item
 * @return T
 */
function wrap(Base $item): Base {
    return $item;
}
===file:App/UseIt.php===
<?php
namespace App;

use Lib\Base;
use Lib\Child;

$child = new Child();
$result = \Lib\wrap($child);
/** @mir-check $result is Lib\Child */
===expect===
