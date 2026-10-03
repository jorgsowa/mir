===description===
A project include reaching into a vendor directory is not followed
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
require_once __DIR__ . '/vendor/lib/x.inc';
function a_fn(): int { return x_fn(); }
===file:vendor/lib/x.inc===
<?php
function x_fn(): int { return 1; }
===expect===
a.module: MixedReturnStatement@3:23-3:37: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@3:30-3:36: Function x_fn() is not defined
