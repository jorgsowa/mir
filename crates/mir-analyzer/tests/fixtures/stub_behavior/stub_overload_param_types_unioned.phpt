===description===
Overloaded stub constructors accept the union of each overload's param types at the same position.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function withEnd(DateTimeInterface $start, DateInterval $step, ?DateTimeInterface $end): void {
    if ($end) {
        $a = new DatePeriod($start, $step, $end);
        /** @mir-check $a is DatePeriod<DateTimeInterface, DateTimeInterface> */
        echo get_class($a);
    }
}

function withRecurrences(DateTimeInterface $start, DateInterval $step): void {
    $b = new DatePeriod($start, $step, 3, DatePeriod::EXCLUDE_START_DATE);
    echo get_class($b);
}

function fromIso(): void {
    $c = new DatePeriod('R5/2023-01-01T00:00:00Z/P1D');
    echo get_class($c);
}

function tooMany(DateTimeInterface $start, DateInterval $step): void {
    $_ = new DatePeriod($start, $step, 3, 0, 1);
    //                                       ^ TooManyArguments: Too many arguments for DatePeriod::__construct(): expected 4, got 5
}
