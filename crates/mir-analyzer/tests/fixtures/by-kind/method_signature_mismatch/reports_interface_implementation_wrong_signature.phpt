===description===
reports interface implementation wrong signature
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {
    public function f(string $x): void;
}
class C implements I {
    public function f(int $x): void { var_dump($x); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method C::f() signature mismatch: parameter $x type 'int' is incompatible with parent type 'string'
}
===expect===
