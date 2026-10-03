===description===
does not report subclass as parent
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Child extends Base {}
function f(Base $x): void { var_dump($x); }
function test(): void { f(new Child()); }
===expect===
