===description===
literal int type parameter is widened so setter accepts other int values
===file:Box.php===
<?php
/**
 * @template T
 */
class Box {
    public $value;
    /**
     * @param T $value
     * @suppress UnusedParam
     */
    public function __construct($value) { $this->value = $value; }
    /**
     * @param T $value
     * @suppress UnusedParam
     */
    public function set($value): void { $this->value = $value; }
}
===file:App.php===
<?php
function app(): void {
    $b = new Box(5);
    $b->set(6);
}
===expect===
Box.php: UnusedPsalmSuppress@11:0-11:0: Suppress annotation for 'UnusedParam' is never used
Box.php: UnusedPsalmSuppress@16:0-16:0: Suppress annotation for 'UnusedParam' is never used
