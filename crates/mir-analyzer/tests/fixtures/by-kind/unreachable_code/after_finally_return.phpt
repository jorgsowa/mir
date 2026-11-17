===description===
after finally return
===file===
<?php
function test(): void {
    try {
        echo 'work';
    } finally {
        return;
//      ^^^^^^^ DeprecatedSyntax: Returning from a finally block is deprecated
    }

    echo 'unreachable';
//  ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
