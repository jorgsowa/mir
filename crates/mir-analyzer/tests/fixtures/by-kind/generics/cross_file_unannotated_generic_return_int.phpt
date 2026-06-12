===description===
cross-file unannotated generic return resolves to int type parameter
===file:box.php===
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
    public function __construct($value) {
        $this->value = $value;
    }
    public function get() { return $this->value; }
}
===file:app.php===
<?php
function app(): void {
    $b = new Box(5);
    $result = $b->get();
    echo $result;
}
===expect===
box.php: UnusedPsalmSuppress@11:0-11:0: Suppress annotation for 'UnusedParam' is never used
