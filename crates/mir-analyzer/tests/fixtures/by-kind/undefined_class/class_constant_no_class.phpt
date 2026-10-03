===description===
Class constant no class
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Ns;

/** @param "foo"|"bar"|C::A|C::B $s */
function foo($s) : void {}
===expect===
