===description===
include_once of a .inc file resolves its function across files when the extension is configured
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
include_once __DIR__ . '/b.inc';
function a_hook(): string { return b_helper(); }
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
===file:orphan.inc===
<?php
function orphan(): int { return 'x'; }
===expect===
