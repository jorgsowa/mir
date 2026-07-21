===description===
`@if-this-is` must be checked on `$var::staticMethod()` through an
object-typed variable too, not just self::/static::/parent::/$this->.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
class Model {
    /** @if-this-is Model<int> */
    public static function onlyIntKeyed(): void {}
}

/** @param Model<string> $m */
function bad(Model $m): void {
    $m::onlyIntKeyed();
}

/** @param Model<int> $m */
function ok(Model $m): void {
    $m::onlyIntKeyed();
}
===expect===
IfThisIsMismatch@10:4-10:22: Cannot call Model::onlyIntKeyed() — @if-this-is requires $this to be 'Model<int>', but it is 'Model<string>'
