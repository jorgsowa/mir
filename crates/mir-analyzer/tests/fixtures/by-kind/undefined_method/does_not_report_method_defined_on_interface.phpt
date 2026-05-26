===description===
does not report method defined on interface
===file===
<?php
interface I {
    public function doIt(): void;
}
function f(I $i): void {
    $i->doIt();
}
===expect===
