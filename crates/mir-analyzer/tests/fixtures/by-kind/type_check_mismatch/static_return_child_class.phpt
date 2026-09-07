===description===
Regression: static return type should resolve to child class when called on child
===config===
suppress=UnusedVariable
===file===
<?php
class Model {
    public static function query(): static { return new static(); }
}
class UserModel extends Model {}
$m = UserModel::query();
/** @mir-check $m is UserModel */
$x = $m;
===expect===
