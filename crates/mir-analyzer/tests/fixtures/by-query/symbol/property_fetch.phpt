===description===
A property fetch resolves to the declaring class's property with its docblock type.
===cursor===
symbol
===file===
<?php
final class User {
    /** @var list<string> */
    public array $roles = [];
}
$u = new User();
$r = $u->ro<CURSOR>les;
===expect===
kind: property User::$roles
type: list<string>
