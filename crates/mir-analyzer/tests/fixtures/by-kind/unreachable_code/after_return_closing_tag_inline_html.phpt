===description===
A closing tag after return is not itself unreachable: the issue lands on the inline HTML.
===file===
<?php
function f(): void {
    return; ?>html<?php
//            ^^^^ UnreachableCode: Unreachable code detected
}
