===description===
`use BaseInit { __construct as __constructBase; }` makes `__constructBase` a real method.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
trait BaseInit {
    public function __construct(int $x) {}
}
class Query {
    use BaseInit { __construct as __constructBase; }
    public function __construct() {
        $this->__constructBase(1);
    }
}
===expect===
