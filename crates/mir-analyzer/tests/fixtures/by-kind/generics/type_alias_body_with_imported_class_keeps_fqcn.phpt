===description===
A @psalm-type body naming imported, sibling, global and nested-alias classes
keeps those names fully qualified at every use site instead of re-prefixing the
consumer's namespace.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:contract.php===
<?php
namespace Lib\Contract;

interface Item {}
===file:consumer.php===
<?php
namespace App\Core;

use Lib\Contract\Item;

class Sibling {}

/**
 * @psalm-type Ctx = array{items: array<string, Item>}
 * @psalm-type Bag = array{item: Item, sib: Sibling, glob: \Countable, cls: class-string<Item>}
 * @psalm-type Nested = list<Ctx>
 * @psalm-type Handler = callable(Item): Sibling
 */
class F {
    /** @var Ctx */
    public array $prop = ['items' => []];

    /** @param Ctx $c */
    public static function viaParam(array $c): void {
        self::find($c['items']);
        /** @mir-check $c is array{'items': array<string, Lib\Contract\Item>} */
        echo 1;
    }

    /** @return Ctx */
    public static function viaReturn(): array { return ['items' => []]; }

    /** @param Bag $m */
    public static function viaBag(array $m): void {
        /** @mir-check $m is array{'item': Lib\Contract\Item, 'sib': App\Core\Sibling, 'glob': Countable, 'cls': class-string<Lib\Contract\Item>} */
        echo 1;
    }

    /** @param Nested $n */
    public static function viaNested(array $n): void {
        /** @mir-check $n is list<array{'items': array<string, Lib\Contract\Item>}> */
        echo 1;
    }

    /** @param Handler $h */
    public static function viaCallable(callable $h): void {
        /** @mir-check $h is callable(Lib\Contract\Item): App\Core\Sibling */
        echo 1;
    }

    public function viaProperty(): void {
        $p = $this->prop;
        /** @mir-check $p is array{'items': array<string, Lib\Contract\Item>} */
        echo 1;
        $r = self::viaReturn();
        /** @mir-check $r is array{'items': array<string, Lib\Contract\Item>} */
        echo 1;
        /** @var Ctx $local */
        $local = $GLOBALS['x'];
        /** @mir-check $local is array{'items': array<string, Lib\Contract\Item>} */
        echo 1;
    }

    /** @param array<string, Item> $i */
    public static function find(array $i): void {}
}
