===description===
Loop with switch doesnt return first case
===file===
<?php
function b(): int {
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
InvalidReturnType@2:19-13:20: Return type 'void' is not compatible with declared 'int'
