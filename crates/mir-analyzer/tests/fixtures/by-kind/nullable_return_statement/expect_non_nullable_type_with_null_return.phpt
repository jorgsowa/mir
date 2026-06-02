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
InvalidReturnType@4:5-4:17: Return type 'null' is not compatible with declared 'Generator'
InvalidReturnType@11:5-11:17: Return type 'null' is not compatible with declared 'Generator'
