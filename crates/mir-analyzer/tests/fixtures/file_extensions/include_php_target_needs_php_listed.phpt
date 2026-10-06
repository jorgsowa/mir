===description===
With php absent from the list, an include of a .php file is not followed
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
require_once __DIR__ . '/b.php';
function a_fn(): int { return b_fn(); }
//                     ^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                            ^^^^^^ UndefinedFunction: Function b_fn() is not defined
===file:b.php===
<?php
function b_fn(): int { return 1; }
