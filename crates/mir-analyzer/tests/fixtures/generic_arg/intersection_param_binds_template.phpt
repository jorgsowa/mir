===description===
intersection parameter type — no InvalidArgument when arg satisfies all parts of the intersection
===file===
<?php
interface Taggable {}

/** @template T */
class Box implements Taggable {}

/**
 * @template T
 * @param Box<T>&Taggable $item
 * @return T
 */
function extract(mixed $item): mixed { return null; }

/** @var Box<string> $b */
$b = new Box();

$val = extract($b);
/** @mir-check $val is string */
===expect===
UnusedParam@12:18: Parameter $item is never used
