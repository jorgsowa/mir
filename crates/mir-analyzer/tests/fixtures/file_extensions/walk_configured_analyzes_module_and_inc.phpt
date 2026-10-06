===description===
Configured extensions are walked: issues surface in .module/.inc files and cross-file calls resolve
===config===
<mir>
  <fileExtensions>
    <extension name="php"/>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:a.module===
<?php
function a_hook(): int { return 'x'; }
//                       ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
function b_bad(): int { return 'x'; }
//                      ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
===file:c.php===
<?php
function c_use(): string { return b_helper(); }
