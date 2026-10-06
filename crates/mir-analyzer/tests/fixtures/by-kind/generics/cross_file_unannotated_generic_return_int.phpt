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
box.php: UnusedSuppress@9:17-9:28: Suppress annotation for 'UnusedParam' is never used
