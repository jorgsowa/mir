===description===
A `do { } while` body always executes at least once, so `$id` is always defined after the loop.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function run(): int {
    do {
        $id = rand(1, 10);
    } while ($id > 5);
    return $id;
}
===expect===
