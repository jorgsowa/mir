===description===
No fatal error on missing class without slash
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Func {
    public function __construct(string $name, callable $callable) {}
}

new Func("f", ["Foo", "bar"]);
===expect===
