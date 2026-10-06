===description===
Calling __construct() on an external object from unserialize() is still flagged — exemption is only for $this.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public function __construct() {}

    public function unserialize(string $data): void {
        $other = new Foo();
        $other->__construct();
//      ^^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of Foo directly
    }
}
