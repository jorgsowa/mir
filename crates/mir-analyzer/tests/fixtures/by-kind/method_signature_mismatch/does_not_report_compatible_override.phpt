===description===
does not report compatible override
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
    public function f(string $x): void { var_dump($x); }
}
===expect===
