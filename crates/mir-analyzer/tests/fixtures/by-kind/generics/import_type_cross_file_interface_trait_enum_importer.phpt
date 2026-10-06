===description===
Interfaces, traits and enums can import a cross-file `@psalm-type` alias; it expands
in method return/param types, properties and constants.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:lib.php===
<?php
namespace Lib;

class Item {}

/** @psalm-type Items = array<int, Item> */
class Box {}
===file:app.php===
<?php
namespace App;

use Lib\Box;

/** @psalm-import-type Items from Box */
interface Repo {
    /** @return Items */
    public function all(): array;
}

/** @psalm-import-type Items from Box */
trait HasItems {
    /** @var Items */
    public array $items = [];
    /** @param Items $i */
    public function take($i): void {
        /** @mir-check $i is array<int, Lib\Item> */
        echo 1;
    }
}

/** @psalm-import-type Items from Box */
enum Kind {
    case A;
    /** @return Items */
    public function items(): array { return []; }
}

function use_them(Repo $r, Kind $k): void {
    $a = $r->all();
    /** @mir-check $a is array<int, Lib\Item> */
    echo 1;
    $b = $k->items();
    /** @mir-check $b is array<int, Lib\Item> */
    echo 1;
}

class UsesTrait {
    use HasItems;
    public function p(): void {
        $p = $this->items;
        /** @mir-check $p is array<int, Lib\Item> */
        echo 1;
    }
}
