===description===
Throwing \Error does not fire InvalidThrow — Error implements Throwable
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
throw new \Error('fatal');
