===description===
Regression: static return type should resolve to the calling class at the call site
===config===
suppress=UnusedVariable
===file===
<?php
class Model {
    public static function query(): static { return new static(); }
}
$m = Model::query();
/** @mir-check $m is Model */
$x = $m;
===expect===
