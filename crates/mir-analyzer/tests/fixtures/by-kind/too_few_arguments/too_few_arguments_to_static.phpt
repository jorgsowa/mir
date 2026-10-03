===description===
Too few arguments to static
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public static function fooFoo(int $a): void {}
}

A::fooFoo();
//<^^^^^^^^^^^ TooFewArguments: Too few arguments for fooFoo(): expected 1, got 0
===expect===
