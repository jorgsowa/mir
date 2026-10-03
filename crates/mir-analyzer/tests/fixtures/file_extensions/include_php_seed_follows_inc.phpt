===description===
A plain .php file including a .inc file follows it
===config===
<mir>
  <projectFiles>
    <file name="index.php"/>
  </projectFiles>
  <fileExtensions>
    <extension name="php"/>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:index.php===
<?php
require_once __DIR__ . '/core.inc';
function boot(): string { return core_name(); }
===file:core.inc===
<?php
function core_name(): string { return 'core'; }
===expect===
