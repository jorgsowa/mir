===description===
cross-file unannotated generic return resolves to int type parameter
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//               ^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UnusedParam' is never used
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
