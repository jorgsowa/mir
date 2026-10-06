===description===
does not report override no type hint on parent
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public function f($x): void { var_dump($x); }
}
class Child extends Base {
    public function f(int $x): void { var_dump($x); }
}
