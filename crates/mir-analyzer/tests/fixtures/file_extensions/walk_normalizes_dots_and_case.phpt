===description===
Entries with a leading dot or mixed case match files case-insensitively, including uppercase file extensions
===config===
<mir>
  <fileExtensions>
    <extension name=".PHP"/>
    <extension name=" Module"/>
    <extension name=".INC"/>
  </fileExtensions>
</mir>
===file:a.MODULE===
<?php
function a_hook(): int { return 'x'; }
//                       ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
===file:lib.Inc===
<?php
function lib_fn(): string { return 'l'; }
===file:c.php===
<?php
function c_use(): string { return lib_fn(); }
