===description===
Explicit invoke type mismatch
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
(new A)->__invoke(1);
//                ^ ArgumentTypeCoercion: Argument $p of __invoke() expects 'string', got '1' — coercion may fail at runtime
===expect===
