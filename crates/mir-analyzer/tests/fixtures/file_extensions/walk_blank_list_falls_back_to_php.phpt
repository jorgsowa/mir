===description===
A blank file_extensions entry normalizes to the default, so only .php is walked
===config===
<mir>
  <fileExtensions>
    <extension name=""/>
    <extension name=" ."/>
  </fileExtensions>
</mir>
===file:a.module===
<?php
function a_hook(): int { return 'x'; }
===file:c.php===
<?php
function c_use(): int { return 'x'; }
//                      ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
===expect===
