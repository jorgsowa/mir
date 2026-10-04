===description===
Loop with switch doesnt return first case
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function b(): int {
//            ^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
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
