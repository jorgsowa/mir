===description===
A `PharData` offset is a `PharFileInfo`.
===file===
<?php
function f(PharData $p): void {
    $e = $p['composer.json'];
    /** @mir-check $e is PharFileInfo */
    echo $e->getContent();
}
