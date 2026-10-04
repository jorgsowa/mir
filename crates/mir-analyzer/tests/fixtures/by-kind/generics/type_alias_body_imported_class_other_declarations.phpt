===description===
Alias bodies naming an imported class keep their FQCN on functions, methods,
interfaces, traits, enums, template bounds and @throws.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:contract.php===
<?php
namespace Lib\Contract;

interface Item {}
class Failure extends \Exception {}
===file:consumer.php===
<?php
namespace App\Core;

use Lib\Contract\Failure;
use Lib\Contract\Item;

/**
 * @psalm-type Row = array{item: Item}
 * @param Row $r
 */
function viaFunction(array $r): void {
    /** @mir-check $r is array{'item': Lib\Contract\Item} */
    echo 1;
}

/** @psalm-type Row = array{item: Item} */
interface I {
    /** @param Row $r */
    public function m(array $r): void;
}

/** @psalm-type Row = array{item: Item} */
trait T {
    /** @param Row $r */
    public function m(array $r): void {
        /** @mir-check $r is array{'item': Lib\Contract\Item} */
        echo 1;
    }
}

/** @psalm-type Row = array{item: Item} */
enum E {
    case A;

    /** @param Row $r */
    public function m(array $r): void {
        /** @mir-check $r is array{'item': Lib\Contract\Item} */
        echo 1;
    }
}

/**
 * @psalm-type Row = array{item: Item}
 * @psalm-type Err = Failure
 * @template TRow of Row
 */
class C {
    /**
     * @psalm-type Local = list<Item>
     * @param Local $l
     * @throws Err
     */
    public function m(array $l): void {
        /** @mir-check $l is list<Lib\Contract\Item> */
        echo 1;
        throw new Failure();
    }
}
===expect===
