===description===
UnhandledMatchCondition does NOT fire for int-valued self:: class constants covering every literal.
===file===
<?php
class Status {
    const OPEN = 1;
    const CLOSED = 2;

    /** @param 1|2 $s */
    public function label(int $s): string {
        return match ($s) {
            self::OPEN => 'open',
            self::CLOSED => 'closed',
        };
    }
}
===expect===
