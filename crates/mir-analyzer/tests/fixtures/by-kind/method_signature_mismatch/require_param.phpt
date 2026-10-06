===description===
Require param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {
    function foo(bool $b = false): void;
}

class C implements I {
    public function foo(bool $b): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method C::foo() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
}
