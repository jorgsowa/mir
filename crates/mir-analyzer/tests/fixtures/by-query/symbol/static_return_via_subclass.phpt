===description===
A `static` return type resolves to the called subclass.
===cursor===
symbol
===file===
<?php
class Model {
    public static function create(): static { return new static(); }
}
final class User extends Model {}
$u = User::cr<CURSOR>eate();
===expect===
kind: static call Model::create
type: User
