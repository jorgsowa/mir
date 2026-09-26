===description===
Find-references with the cursor on a declaration's name resolves that declaration.
===ignore===
===cursor===
references include_declaration
===file===
<?php
final class Greeter { public function gr<CURSOR>eet(): void {} }
(new Greeter())->greet();
===expect===
test.php@2:38-2:43
test.php@3:17-3:22
