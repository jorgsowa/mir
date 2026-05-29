===description===
Array key type preserved with class-string key ($arr[$cls] = $value)
===file===
<?php

class Event {}
class EventSubscriber {}

/**
 * @param array<class-string<Event>, callable> $listeners
 * @return array<class-string<Event>, callable>
 */
function buildMap(array $listeners): array
{
    $result = [];
    foreach ($listeners as $cls => $callback) {
        $result[$cls] = $callback;
    }
    return $result;
}
===expect===
