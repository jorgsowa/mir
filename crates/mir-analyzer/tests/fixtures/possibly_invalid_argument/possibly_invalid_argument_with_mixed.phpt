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
PossiblyInvalidArgument
===ignore===
TODO
