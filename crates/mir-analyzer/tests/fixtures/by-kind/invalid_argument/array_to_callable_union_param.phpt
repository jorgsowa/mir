===description===
An array passed to a `callable|array|null` union param (Http Factory::fake
pattern) matches the array alternative — it must not be validated as an
[object, "method"] callable shape. A pure callable param still validates.
===file===
<?php
class Factory {
    /**
     * @param  callable|array|null  $callback
     */
    public function fake($callback = null): static { return $this; }
}

function pure_callable(callable $cb): void {}

$f = new Factory();
$f->fake([
    '*' => ['result' => ['foo' => 'bar']],
]);
pure_callable(['only-one-element']);
===expect===
InvalidArgument@15:15-15:35: Argument $callback of callable() expects 'callable (string or [object, "method"])', got 'array{0: "only-one-element"}'
