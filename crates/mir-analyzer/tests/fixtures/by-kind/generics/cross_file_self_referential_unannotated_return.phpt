===description===
cross-file self-referential unannotated return falls back without hanging
===file:rec.php===
<?php
/**
 * @template T
 */
class Rec {
    public $value;
    /**
     * @param T $value
     * @suppress UnusedParam
     */
    public function __construct($value) {
        $this->value = $value;
    }
    public function loop() { return $this->loop(); }
}
===file:app.php===
<?php
function app(): void {
    $r = new Rec(1);
    echo $r->loop();
}
===expect===
