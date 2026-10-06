===description===
$this->__construct() in a helper called from the legacy Serializable::unserialize() still fires (exemption is method-direct only)
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function __construct() {}
    public function init(): void {
        $this->__construct();
//      ^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of A directly
    }
    public function unserialize(string $data): void {
        $this->init();
    }
}
