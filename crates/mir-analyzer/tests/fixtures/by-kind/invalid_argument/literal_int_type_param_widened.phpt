===description===
literal int type parameter is widened so setter accepts other int values
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//               ^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UnusedParam' is never used
     */
    public function __construct($value) { $this->value = $value; }
    /**
     * @param T $value
     * @suppress UnusedParam
//               ^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UnusedParam' is never used
     */
    public function set($value): void { $this->value = $value; }
}
===file:App.php===
<?php
function app(): void {
    $b = new Box(5);
    $b->set(6);
}
