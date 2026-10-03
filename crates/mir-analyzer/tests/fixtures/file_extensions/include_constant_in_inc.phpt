===description===
A constant declared in an included .inc file resolves
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
require_once __DIR__ . '/consts.inc';
function a_fn(): int { return LIMIT; }
function a_str(): string { return LIMIT; }
===file:consts.inc===
<?php
const LIMIT = 10;
===expect===
a.module: InvalidReturnType@4:27-4:40: Return type '10' is not compatible with declared 'string'
