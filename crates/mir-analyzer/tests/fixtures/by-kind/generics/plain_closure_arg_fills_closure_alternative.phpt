===description===
A plain `Closure` argument fills a `Closure(...): T` or `callable` alternative of a template parameter, so `T` binds only the rest of the argument.
===file===
<?php
interface Helpers {
    /**
     * @template TValue
     * @param TValue|\Closure(): TValue $value
     * @return TValue
     */
    public function value($value);

    /**
     * @template T
     * @param T|callable(): T $value
     * @return T
     */
    public function call($value);
}

/** @param \Closure|string $view */
function closureOrString(Helpers $h, $view): void {
    $a = $h->value($view);
    /** @trace $a */
    echo $a;
//  ^^^^^^^^ Trace: Type of $a is string
    $b = $h->call($view);
    /** @trace $b */
    echo $b;
//  ^^^^^^^^ Trace: Type of $b is string
}

function onlyClosure(Helpers $h, \Closure $view): void {
    $c = $h->value($view);
//  ^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $c is assigned a mixed type
    /** @trace $c */
    echo $c;
//  ^^^^^^^^ Trace: Type of $c is mixed
}

function noClosure(Helpers $h, int $view): void {
    $d = $h->value($view);
    /** @trace $d */
    echo $d;
//  ^^^^^^^^ Trace: Type of $d is int
}
