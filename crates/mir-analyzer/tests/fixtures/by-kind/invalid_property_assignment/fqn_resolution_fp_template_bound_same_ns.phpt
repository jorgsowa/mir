===description===
FP guard: template bound — InvalidTemplateParam for unresolved bare bound is pre-existing, NOT caused by this fix.
Template bounds (`of Base`) are stored without FQN resolution since before this change; the fix only
affects return/param type resolution. Documented here as a known pre-existing issue.
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
UseIt.php: InvalidTemplateParam@8:11: Template type 'T' inferred as 'Lib\Child' does not satisfy bound 'Base'
