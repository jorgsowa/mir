===description===
reports override adds required param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public function f(): void {}
}
class Child extends Base {
    public function f(string $x): void { var_dump($x); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::f() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
}
===expect===
