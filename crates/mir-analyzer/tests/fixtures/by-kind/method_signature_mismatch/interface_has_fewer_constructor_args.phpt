===description===
Interface has fewer constructor args
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Foo {
    public function __construct();
}

class Bar implements Foo {
    public function __construct(bool $foo) {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Bar::__construct() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
}
