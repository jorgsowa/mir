===description===
Constant paths, and tainted values in non-path positions, are not reported.
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function test(): void {
    mkdir('/tmp/x');
    copy('/tmp/a', $_GET['dest']);
    chmod('/tmp/a', 0644);
}
===expect===
