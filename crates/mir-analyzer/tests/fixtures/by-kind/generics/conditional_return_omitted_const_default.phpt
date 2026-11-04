===description===
An omitted argument defaulting to `self::CONST` binds the template, so `(B is 0|1 ? A : B)` resolves
===file===
<?php
interface Container {
    public const EXCEPTION = 1;
    public const NULL = 2;

    /**
     * @template B of self::*
     * @param B $behavior
     * @return (B is 0|1 ? object : object|null)
     */
    public function get(string $id, int $behavior = self::EXCEPTION): ?object;
}

function f(Container $c): object {
    return $c->get('x');
}
