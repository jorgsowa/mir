===description===
A property fetch inside string interpolation counts as a reference.
===cursor===
references
===file===
<?php
final class User {
    public string $name = 'x';
}
$u = new User();
echo "Hi {$u->name}";
echo $u->na<CURSOR>me;
===expect===
test.php@6:14-6:18
test.php@7:9-7:13
