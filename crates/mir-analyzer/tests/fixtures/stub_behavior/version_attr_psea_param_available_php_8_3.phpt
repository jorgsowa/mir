===description===
PhpStormStubsElementAvailable: strrchr() third param available on PHP 8.3 — three args accepted
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
$x = strrchr("hello", "l", true);
===expect===
