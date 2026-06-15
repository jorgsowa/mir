===description===
intersection with concrete part first (Taggable&Box<T>) — order should not affect suppression or binding
===file===
<?php
interface Taggable {}

/** @template T */
class Box implements Taggable {}

/**
 * @template T
 * @param Taggable&Box<T> $item
 * @return T
 */
function extract(mixed $item): mixed { return null; }

/** @var Box<string> $b */
$b = new Box();

$val = extract($b);
/** @mir-check $val is string */
echo $val;
===expect===
UnusedParam@12:17-12:28: Parameter $item is never used
