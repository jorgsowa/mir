===description===
dirname(__FILE__, 2) does not resolve to the including file's own directory
===config===
<mir>
  <projectFiles>
    <file name="deep/a.module"/>
  </projectFiles>
  <fileExtensions>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:deep/a.module===
<?php
require dirname(__FILE__, 2) . '/sibling.inc';
function a_hook(): int { return sibling(); }
//                       ^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                              ^^^^^^^^^ UndefinedFunction: Function sibling() is not defined
===file:deep/sibling.inc===
<?php
function sibling(): int { return 1; }
===expect===
