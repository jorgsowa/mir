===description===
The check abstains entirely — rather than risk a false positive — once
`$this` may have reached a call this analysis can't see into: calling any
other (unproven) method on `$this`, even one unrelated to initialization,
or delegating to `parent::__construct()`. Both are common real-world
patterns (init helpers, constructor delegation) that this pass can't verify
actually assign the property, so it stays silent rather than guess wrong.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public string $inherited;

    public function __construct() {
        $this->inherited = 'base';
    }
}

class DelegatesToHelper {
    public string $value;
    private int $created = 0;

    public function __construct(string $v) {
        $this->logCreation();
    }

    private function logCreation(): void {
        $this->created++;
    }
}

class DelegatesToParent extends Base {
    public string $value;

    public function __construct(string $v) {
        parent::__construct();
    }
}
