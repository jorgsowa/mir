===description===
Class names in a `@psalm-type` body resolve once against the declaring
namespace (no `A\B\A\B\Item`) in every position that expands the alias.
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment,MixedArrayAccess
===file===
<?php
namespace A\B;

class Item {}
class Sub {}

/**
 * @psalm-type Items = array<int, Item>
 * @psalm-type Nested = list<\A\B\Item>
 * @psalm-type Rel = Sub|null
 */
class Box {
    /** @var Items */
    public array $p = [];

    /**
     * @psalm-type Local = Item|null
     * @param Local $l
     * @param Nested $n
     * @param Rel $r
     * @return Items
     */
    public function f($l, $n, $r) {
        /** @mir-check $l is A\B\Item|null */
        echo 1;
        /** @mir-check $n is list<A\B\Item> */
        echo 1;
        /** @mir-check $r is A\B\Sub|null */
        echo 1;
        /** @var Items $v */
        $v = $GLOBALS['x'];
        /** @mir-check $v is array<int, A\B\Item> */
        echo 1;
        $c = /** @return Items */ function () { return []; };
        $x = $c();
        /** @mir-check $x is array<int, A\B\Item> */
        echo 1;
        $y = $this->p;
        /** @mir-check $y is array<int, A\B\Item> */
        echo 1;
        return [];
    }
}

/**
 * @psalm-type MI = Item|null
 * @param MI $i
 * @return MI
 */
function fx($i) {
    /** @mir-check $i is A\B\Item|null */
    echo 1;
    return $i;
}
$o = fx(null);
/** @mir-check $o is A\B\Item|null */
echo 1;
===expect===
