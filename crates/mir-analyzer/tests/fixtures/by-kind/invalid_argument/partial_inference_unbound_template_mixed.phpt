===description===
partial type parameter inference leaves unbound template as mixed, not fabricated to bound
===config===
suppress=MissingPropertyType
===file:Map.php===
<?php
class Base {}
class Other {}
/**
 * @template K
 * @template V of Base
 */
class Map {
    private $key;
    private $value;
    /** @param K $key */
    public function __construct($key) { $this->key = $key; }
    /** @param V $value */
    public function put($value): void { $this->value = $value; }
}
===file:App.php===
<?php
function app(): void {
    $m = new Map("k");
    $m->put(new Other());
}
===expect===
