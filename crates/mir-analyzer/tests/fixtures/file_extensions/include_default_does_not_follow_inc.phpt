===description===
With the default extension list an included .inc target is not followed, so its function is undefined
===config===
<mir>
  <projectFiles>
    <file name="a.module"/>
  </projectFiles>
</mir>
===file:a.module===
<?php
include_once __DIR__ . '/b.inc';
function a_hook(): string { return b_helper(); }
//                          ^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
//                                 ^^^^^^^^^^ UndefinedFunction: Function b_helper() is not defined
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
function b_bad(): int { return 'x'; }
===expect===
