===description===
cross-file self-referential unannotated return falls back without hanging
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//               ^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UnusedParam' is never used
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
