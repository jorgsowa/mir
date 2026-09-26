===description===
Variables are not indexed symbols, so find-references resolves nothing.
===cursor===
references
===file===
<?php
$name = 'x';
echo $na<CURSOR>me;
===expect===
error: NotFound
