===description===
$this->__construct() inside __clone is a valid re-initialization pattern and must not emit DirectConstructorCall.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Registry {
    private array $items = [];

    public function __construct(private string $name) {}

    public function __clone(): void {
        $this->__construct($this->name . '_copy');
    }
}
===expect===
