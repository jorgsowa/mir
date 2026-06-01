===description===
bare Closure type satisfies Closure(): T parameter — no InvalidArgument
===file===
<?php
class PDO {}

/**
 * @param PDO|Closure(): PDO $pdo
 */
function connect(PDO|Closure $pdo): PDO {
    if ($pdo instanceof Closure) {
        return $pdo();
    }
    return $pdo;
}

/** @var PDO|Closure $connection */
$connection = new PDO();

connect($connection);
===expect===
