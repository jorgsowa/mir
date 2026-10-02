===description===
References from a function declaration name include its calls.
===cursor===
references include_declaration
===file===
<?php
function ba<CURSOR>r(): void {}
bar();
===expect===
test.php@2:9-2:12
test.php@3:0-3:3
