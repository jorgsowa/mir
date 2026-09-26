===description===
Hover on a variable shows its inferred type at that point.
===cursor===
hover
===file===
<?php
function f(?int $n): void {
    if ($n !== null) {
        echo $<CURSOR>n;
    }
}
===expect===
type: int
