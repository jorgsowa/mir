===description===
Possibly invalid argument with mixed
===file===
<?php declare(strict_types=1);
/**
 * @suppress MissingParamType
 * @suppress MixedArgument
 */
function foo($a) : void {
    if (rand(0, 1)) {
        $a = 0;
    }

    echo strlen($a);
}
===expect===
UnusedPsalmSuppress@6:0-6:0: Suppress annotation for 'MixedArgument' is never used
MixedArgument@11:16-11:18: Argument $string of strlen() is mixed
