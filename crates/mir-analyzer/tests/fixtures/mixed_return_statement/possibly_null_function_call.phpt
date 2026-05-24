===description===
possiblyNullFunctionCall
===file===
<?php
/**
 * @var Closure|null $foo
 */
$foo = null;


$foo =
    /**
     * @param mixed $bar
     * @psalm-suppress MixedFunctionCall
     */
    function ($bar) use (&$foo): string
    {
        if (is_array($bar)) {
            return $foo($bar);
        }

        return $bar;
    };
===expect===
MixedReturnStatement
===ignore===
TODO
