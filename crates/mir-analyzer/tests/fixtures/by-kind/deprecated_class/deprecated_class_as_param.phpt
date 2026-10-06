===description===
Deprecated class as param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @deprecated
 */
class DeprecatedClass{}

function foo(DeprecatedClass $deprecatedClass): void {}
//           ^^^^^^^^^^^^^^^ DeprecatedClass: Class DeprecatedClass is deprecated
