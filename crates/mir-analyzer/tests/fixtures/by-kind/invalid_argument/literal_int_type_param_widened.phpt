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
Box.php: UnusedSuppress@9:17-9:28: Suppress annotation for 'UnusedParam' is never used
Box.php: UnusedSuppress@14:17-14:28: Suppress annotation for 'UnusedParam' is never used
