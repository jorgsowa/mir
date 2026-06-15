===description===
Expect non nullable type with null return
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
InvalidReturnType@4:4-4:16: Return type 'null' is not compatible with declared 'Generator'
InvalidReturnType@11:4-11:16: Return type 'null' is not compatible with declared 'Generator'
