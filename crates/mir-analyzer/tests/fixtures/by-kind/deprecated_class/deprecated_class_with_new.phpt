===description===
Deprecated class with new
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @deprecated
 */
class Foo { }

$a = new Foo();
//       ^^^ DeprecatedClass: Class Foo is deprecated
===expect===
