===description===
`is_string($x ??= ...)` narrows `$x` on both branches
===file===
<?php
final class Clock {
    private \DateTimeZone $tz;

    public function __construct(\DateTimeZone|string|null $tz = null) {
        $this->tz = is_string($tz ??= date_default_timezone_get()) ? new \DateTimeZone($tz) : $tz;
    }
}
