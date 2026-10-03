===description===
UndefinedDocblockClass fires when a @param docblock names a class that does
not exist, even without a native type hint.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param NonExistentParamClass $x
 */
function process($x): void {}
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentParamClass' does not exist

===expect===
