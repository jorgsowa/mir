===description===
Deprecated class with new attr
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
#[\Deprecated]
class Foo { }

$a = new Foo();
//       ^^^ DeprecatedClass: Class Foo is deprecated
===expect===
