===description===
Array key type preserved when pushing into nested array ($arr[$cls][] = $item)
===file===
<?php

class Event {}

/**
 * @param array<class-string<Event>, callable> $listeners
 * @return array<class-string<Event>, list<callable>>
 */
function buildGrouped(array $listeners): array
{
    $out = [];
    foreach ($listeners as $cls => $callback) {
        $out[$cls][] = $callback;
    }
    return $out;
}
===expect===
