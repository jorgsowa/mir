===description===
Throwing \TypeError (extends \Error) does not fire InvalidThrow
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
throw new \TypeError('type mismatch');
