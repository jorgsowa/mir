===description===
variadic union nullable ctor args do not panic or cause false positive
===file:Coll.php===
<?php
class Base {}
class Other {}
/**
 * @template T
 * @template U of Base
 */
class Coll {
    private $items;
    private $value;
    /**
     * @param T ...$items
     * @suppress UnusedParam
     */
    public function __construct(?string $label, int|string $tag, ...$items) {
        $this->items = $items;
        if ($label !== null) echo $label;
        echo $tag;
    }
    /** @param U $value */
    public function setValue($value): void { $this->value = $value; }
}
===file:App.php===
<?php
function app(): void {
    $c = new Coll(null, 7, 1, 2, 3);
    $c->setValue(new Other());
}
===expect===
Coll.php: UnusedPsalmSuppress@15:0-15:0: Suppress annotation for 'UnusedParam' is never used
