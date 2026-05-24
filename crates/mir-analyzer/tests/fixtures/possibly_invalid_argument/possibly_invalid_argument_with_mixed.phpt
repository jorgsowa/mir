===description===
possiblyInvalidArgumentWithMixed
===file===
<?php declare(strict_types=1);
/**
 * @psalm-suppress MissingParamType
 * @psalm-suppress MixedArgument
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
