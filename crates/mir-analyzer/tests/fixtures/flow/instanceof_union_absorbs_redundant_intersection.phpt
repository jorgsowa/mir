===file===
<?php
class R { public function r(): void {} }
interface I {}

function f(R|I $c): void {
    if ($c instanceof R) {
        /** @mir-check $c is R */
        echo 1;
    }
}
