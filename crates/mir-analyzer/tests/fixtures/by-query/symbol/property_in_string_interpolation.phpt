===description===
A property fetch inside string interpolation resolves to the property.
===cursor===
symbol
===file===
<?php
final class User {
    public string $name = 'x';
}
$u = new User();
echo "Hi {$u->na<CURSOR>me}";
===expect===
kind: property User::$name
type: string
