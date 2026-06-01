===description===
Variable assigned before try and read only in finally block is not reported as unused
===file===
<?php
function withoutTablePrefix(callable $callback): mixed {
    $tablePrefix = 'prefix_';
    echo '';
    try {
        return $callback();
    } finally {
        echo $tablePrefix;
    }
}
===expect===
