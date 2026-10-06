===description===
A target whose extension is not configured breaks the chain even when earlier links are followed
===config===
<mir>
  <projectFiles>
    <file name="a.module"/>
  </projectFiles>
  <fileExtensions>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:a.module===
<?php
require_once __DIR__ . '/b.inc';
function a_hook(): int { return t(); }
//                       ^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                              ^^^ UndefinedFunction: Function t() is not defined
===file:b.inc===
<?php
require_once __DIR__ . '/c.tpl';
===file:c.tpl===
<?php
function t(): int { return 1; }
===expect===
