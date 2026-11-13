===description===
`$obj::CONST` on an object of an unknown class reports only the unknown class.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(Missing\Client $c): string { return $c::VERSION; }
//         ^^^^^^^^^^^^^^ UndefinedClass: Class Missing\Client does not exist
