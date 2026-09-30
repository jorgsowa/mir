===description===
Loop with switch doesnt return first case
===config===
suppress=MissingThrowsDocblock,UnusedForeachValue
===file===
<?php
function b(): int {
//                ^ +11:1 InvalidReturnType: Return type 'void' is not compatible with declared 'int'
    switch (random_int(1, 10)) {
        case 1:
            foreach([1,2] as $i) {
                continue;
            }
            break;

        default:
            return 2;
    }
}
===expect===
