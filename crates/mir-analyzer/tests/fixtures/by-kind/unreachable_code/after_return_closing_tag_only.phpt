===description===
A closing tag after return with no code following is not unreachable code.
===file===
<?php
function f(): void {
    return;
    ?>
<?php
}
