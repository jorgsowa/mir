===cursor===
hover
===file===
<?php
final class User {
    /** @var list<string> */
    public array $roles = [];
}
$u = new User();
$r = $u->ro<CURSOR>les;
===expect===
type: list<string>
definition: test.php@4:4-4:28
