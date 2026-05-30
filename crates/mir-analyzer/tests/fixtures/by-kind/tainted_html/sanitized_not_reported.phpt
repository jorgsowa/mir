===description===
sanitized not reported
===file===
<?php
function test(): void {
    echo htmlspecialchars($_GET['x']);
}
===expect===
