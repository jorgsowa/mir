===description===
new with mixed variable should not error (already imprecise)
===file===
<?php
function test(mixed $value) {
    new $value();
}
===expect===
