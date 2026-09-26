===description===
References to a built-in function list project calls only, never the stub declaration.
===cursor===
references include_declaration
===file:a.php===
<?php
echo str<CURSOR>len('a');
===file:b.php===
<?php
namespace App;

echo strlen('b') + \strlen('c');
===expect===
a.php@2:5-2:11
b.php@4:5-4:11
b.php@4:19-4:26
