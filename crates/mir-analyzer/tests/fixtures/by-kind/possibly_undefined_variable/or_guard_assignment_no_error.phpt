===description===
no PossiblyUndefinedVariable when var is assigned in || RHS and guard returns on true branch
===file===
<?php
function loadSchema(mixed $connection): void {
    // $path is assigned in the RHS of ||; after the guard returns, $path is definitely assigned
    if (! ($connection instanceof \stdClass) ||
        ! is_file($path = '/tmp/' . get_class($connection))) {
        return;
    }

    echo $path;
}
===expect===
