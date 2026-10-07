===description===
DateTime/DateTimeImmutable withers return `static`, never `false`, so their result chains and
passes as the object type.
===file===
<?php
final class Day {
    public function __construct(public DateTimeImmutable $at) {}
}

function startOfDay(DateTimeImmutable $at, DateTime $mutable): Day {
    $start = $at->setTime(0, 0);
    /** @mir-check $start is DateTimeImmutable */
    $date = $mutable->setDate(2024, 1, 1)->setTime(hour: 0, minute: 0);
    /** @mir-check $date is DateTime */
    $seconds = $at->getTimestamp() - $date->getTimestamp();
    /** @mir-check $seconds is int */
    return new Day($start->setTimestamp($seconds));
}
