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
===file:b.inc===
<?php
function b_fn(): int { return 1; }
===expect===
a.module: MixedReturnStatement@4:23-4:49: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@4:39-4:48: Function nope_fn() is not defined
