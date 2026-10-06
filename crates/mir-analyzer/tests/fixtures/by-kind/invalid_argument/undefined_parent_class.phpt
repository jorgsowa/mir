===description===
Undefined parent class
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @suppress UndefinedClass
 */
class B extends A {}

$b = new B();
