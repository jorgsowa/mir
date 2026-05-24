===description===
expectNonNullableTypeWithNullReturn
===file===
<?php
function example() : Generator {
    yield from [2];
    return null;
}

function example2() : Generator {
    if (rand(0, 1)) {
        return example();
    }
    return null;
}
===expect===
NullableReturnStatement
===ignore===
TODO
