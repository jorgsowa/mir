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
UnusedPsalmSuppress@13:0-13:0: Suppress annotation for 'MixedFunctionCall' is never used
