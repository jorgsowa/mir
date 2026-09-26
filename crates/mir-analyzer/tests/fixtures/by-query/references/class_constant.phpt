===cursor===
references
===file===
<?php
final class Status {
    public const ACTIVE = 1;
    public function isActive(int $s): bool { return $s === self::ACTIVE; }
}
echo Status::ACT<CURSOR>IVE;
===expect===
test.php@4:65-4:71
test.php@6:13-6:19
