===file===
<?php
interface I {
    public function doIt(): void;
}
function f(I $i): void {
    $i->doIt();
}
===expect===
