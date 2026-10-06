===description===
Names in Closure(...)/callable(...) signatures of an inline @var resolve through use imports
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:lib.php===
<?php
namespace Lib;
class Row {}
class Other {}
===file:app.php===
<?php
namespace App\Svc;

use Lib\Row;
use Lib\Other;
use Lib\Row as Aliased;

function run(callable $c): void {
    /** @var \Closure(Row): Other $ret */
    $ret = $c;
    $a = $ret(new Row());
    /** @mir-check $a is Lib\Other */
    echo 1;

    /** @var \Closure(Aliased): list<Other> $nested */
    $nested = $c;
    $b = $nested(new Row());
    /** @mir-check $b is list<Lib\Other> */
    echo 1;

    /** @var callable(Row): Other $cb */
    $cb = $c;
    $d = $cb(new Row());
    /** @mir-check $d is Lib\Other */
    echo 1;

    /** @var Other|(\Closure(Row): Row) $union */
    $union = $c;
    /** @mir-check $union is Lib\Other|Closure(Lib\Row): Lib\Row */
    echo 1;
}
