===description===
A function-level cross-file `@psalm-import-type` expands in `@param`, `@return`,
local `@var`, and the call-site result.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:lib.php===
<?php
namespace Lib;

class Item {}

/**
 * @psalm-type Items = array<int, Item>
 * @psalm-type Shape = array{item: Item, n: int}
 */
class Box {}
===file:app.php===
<?php
namespace App;

use Lib\Box;

/**
 * @psalm-import-type Items from Box
 * @psalm-import-type Shape as S from \Lib\Box
 * @param Items $items
 * @param S $s
 * @return Items
 */
function pick($items, $s) {
    /** @mir-check $items is array<int, Lib\Item> */
    echo 1;
    /** @mir-check $s is array{'item': Lib\Item, 'n': int} */
    echo 1;
    /** @var Items $local */
    $local = $GLOBALS['x'];
    /** @mir-check $local is array<int, Lib\Item> */
    echo 1;
    return $items;
}

$r = pick([], ['item' => new \Lib\Item(), 'n' => 1]);
/** @mir-check $r is array<int, Lib\Item> */
echo 1;
===expect===
