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
//                     ^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                            ^^^^^^ UndefinedFunction: Function x_fn() is not defined
===file:vendor/lib/x.inc===
<?php
function x_fn(): int { return 1; }
