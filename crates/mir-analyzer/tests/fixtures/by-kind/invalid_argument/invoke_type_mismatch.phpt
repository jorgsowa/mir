===description===
Invoke type mismatch
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}

$q = new A;
$q(1);
// ^ ArgumentTypeCoercion: Argument $p of A::__invoke() expects 'string', got '1' — coercion may fail at runtime
