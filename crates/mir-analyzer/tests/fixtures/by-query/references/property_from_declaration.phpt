===description===
Find-references from a property declaration name lists the declaration and its uses.
===cursor===
references include_declaration
===file===
<?php
class User { public string $na<CURSOR>me = ""; }
echo (new User)->name;
===expect===
test.php@2:28-2:32
test.php@3:17-3:21
