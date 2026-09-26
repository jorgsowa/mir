===description===
A cursor on `->` resolves to the receiver's type, which member completion reads.
===cursor===
symbol
===file===
<?php
final class User {
    public array $roles = [];
}
$u = new User();
$r = $u-<CURSOR>>roles;
===expect===
kind: receiver
type: User
