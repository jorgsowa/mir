===description===
@psalm-self-out retypes the receiver variable to the declared type after the
call returns.
===config===
suppress=UnusedParam
===file===
<?php
class MaybeString {
    /** @psalm-self-out ReadyString */
    public function withValue(string $v): void {}
}
class ReadyString extends MaybeString {}

$m = new MaybeString();
$m->withValue("hi");
/** @mir-check $m is ReadyString */
$_ = 1;
===expect===
