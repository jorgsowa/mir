===description===
does not report correct interface implementation
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
    public function f(string $x): void { var_dump($x); }
}
