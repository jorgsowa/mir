===description===
When vendor is listed in projectFiles an include target under vendor joins the closure
===config===
<mir>
  <projectFiles>
    <file name="a.module"/>
    <directory name="vendor"/>
  </projectFiles>
  <fileExtensions>
    <extension name="module"/>
    <extension name="inc"/>
  </fileExtensions>
</mir>
===file:a.module===
<?php
require __DIR__ . '/vendor/pkg/helper.inc';
function a_hook(): int { return pkg_helper(); }
===file:vendor/pkg/helper.inc===
<?php
function pkg_helper(): int { return 1; }
