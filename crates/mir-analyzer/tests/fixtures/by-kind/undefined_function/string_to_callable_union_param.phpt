===description===
A plain string passed to a `(callable)|TValue|string` union param (Collection
::max / ::contains pattern) matches the non-callable alternatives — it must
not be validated as a function name. A pure `callable` param still validates.
===file===
<?php
/**
 * @template TValue
 */
class Col {
    /**
     * @param (callable(TValue): mixed)|string|null $callback
     */
    public function max($callback = null) { return null; }

    /**
     * @param (callable(TValue): bool)|TValue|string $key
     */
    public function contains($key, mixed $value = null): bool { return false; }
}

function pure_callable(callable $cb): void {}

$c = new Col();
$c->max('foo');
$c->contains('id', 1);
pure_callable('not_a_real_function');
===expect===
UndefinedFunction@22:15-22:36: Function not_a_real_function() is not defined
