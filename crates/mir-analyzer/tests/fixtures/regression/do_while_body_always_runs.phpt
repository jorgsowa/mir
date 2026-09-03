===description===
A `do { } while` body always executes at least once, so `$id` is always defined after the loop.
===config===
php_version=8.4
===file===
<?php
function run(): int {
    do {
        $id = rand(1, 10);
    } while ($id > 5);
    return $id;
}
===expect===
