===description===
Mismatching interface method signature
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A {
    public function fooFoo(int $a): void;
}

class B implements A {
    public function fooFoo(string $a): void {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foofoo() signature mismatch: parameter $a type 'string' is incompatible with parent type 'int'

    }
}
