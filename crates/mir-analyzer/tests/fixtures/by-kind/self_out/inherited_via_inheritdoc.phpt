===description===
@psalm-self-out is inherited through @inheritDoc, the same way return type,
params, and throws already are — an override that redeclares no docblock of
its own still retypes the receiver.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /** @psalm-self-out Ready */
    public function touch(): void {}
}
class Ready extends Base {}

class Sub extends Base {
    /** @inheritDoc */
    public function touch(): void {}
}

$x = new Sub();
$x->touch();
/** @mir-check $x is Ready */
$_ = 1;
===expect===
