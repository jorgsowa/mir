===description===
cross-file unannotated generic return with explicit @var property resolves correctly
===file:holder.php===
<?php
/**
 * @template T
 */
class Holder {
    /** @var T */
    public $value;
    /**
     * @param T $v
     * @suppress UnusedParam
     */
    public function __construct($v) { $this->value = $v; }
    public function get() { return $this->value; }
}
===file:app.php===
<?php
function app(): void {
    $h = new Holder(42);
    $result = $h->get();
    echo $result;
}
===expect===
holder.php: UnusedPsalmSuppress@12:0-12:0: Suppress annotation for 'UnusedParam' is never used
