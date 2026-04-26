===file:Clock.php===
<?php
function now(): \DateTimeImmutable {
    return new \DateTimeImmutable();
}
===file:Main.php===
<?php
$dt = now();
$formatted = $dt->format('Y-m-d');
===expect===
