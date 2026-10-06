===description===
An include of a nonexistent file is skipped without failing the walk
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
require_once __DIR__ . '/nope.inc';
require_once __DIR__ . '/b.inc';
function a_fn(): int { return b_fn() + nope_fn(); }
//                     ^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                                     ^^^^^^^^^ UndefinedFunction: Function nope_fn() is not defined
===file:b.inc===
<?php
function b_fn(): int { return 1; }
===expect===
