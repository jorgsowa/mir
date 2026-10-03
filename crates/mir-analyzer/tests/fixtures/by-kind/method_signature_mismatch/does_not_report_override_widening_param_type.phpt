===description===
does not report override widening param type
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public function f(string $x): void { var_dump($x); }
}
class Child extends Base {
    public function f(string|int $x): void { var_dump($x); }
}
===expect===
