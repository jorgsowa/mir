===description===
Different argument types
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a, int $b): void {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foofoo() signature mismatch: parameter $b type 'int' is incompatible with parent type 'bool'

    }
}
===expect===
