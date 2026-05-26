===description===
Possibly null function call
===file===
<?php
/**
 * @var Closure|null $foo
 */
$foo = null;


$foo =
    /**
     * @param mixed $bar
     * @suppress MixedFunctionCall
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
