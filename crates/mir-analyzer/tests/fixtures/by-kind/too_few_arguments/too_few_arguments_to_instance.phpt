===description===
Too few arguments to instance
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function fooFoo(int $a): void {}
}

(new A)->fooFoo();
//<^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for fooFoo(): expected 1, got 0
