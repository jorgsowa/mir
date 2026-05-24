===description===
shouldWarnAboutNoGeneratorReturn
===file===
<?php
function generator2() : Generator {
    if (rand(0,1)) {
        return;
    }
    yield 2;
}

/**
 * @psalm-suppress InvalidNullableReturnType
 */
function notagenerator() : Generator {
    if (rand(0, 1)) {
        return;
    }
    return generator2();
}
===expect===
InvalidReturnStatement
===ignore===
TODO
