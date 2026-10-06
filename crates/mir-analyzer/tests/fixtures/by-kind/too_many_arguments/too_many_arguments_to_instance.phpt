===description===
Too many arguments to instance
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

(new A)->fooFoo(5, "dfd");
//                 ^^^^^ TooManyArguments: Too many arguments for fooFoo(): expected 1, got 2
