===description===
new with mixed variable is not InvalidStringClass — mixed is already imprecise
(a Mixed* concern), matching the static-call path
===file===
<?php
function test(mixed $value) {
    new $value();
}
===expect===
