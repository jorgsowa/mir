===description===
A `use`-imported short class name nested inside `array<int, ShortName>` in a @param docblock must resolve against the import, not fire UndefinedDocblockClass.
===config===
suppress=UnusedParam
===file:Item.php===
<?php
namespace App\Model;
final class Item {}
===file:main.php===
<?php
namespace App;
use App\Model\Item;

/**
 * @param array<int, Item> $items
 */
function process($items): void {}
===expect===
