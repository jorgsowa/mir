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
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
function b_bad(): int { return 'x'; }
===expect===
a.module: MixedReturnStatement@3:28-3:46: Cannot return a mixed type from function with declared return type 'string'
a.module: UndefinedFunction@3:35-3:45: Function b_helper() is not defined
