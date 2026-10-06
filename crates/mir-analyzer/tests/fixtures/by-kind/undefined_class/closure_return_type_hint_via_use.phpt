===description===
closure return type hint via use
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(): Foo {};
//                ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
