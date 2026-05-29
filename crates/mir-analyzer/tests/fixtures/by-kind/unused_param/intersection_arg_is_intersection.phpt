===description===
arg is itself an intersection type satisfying the concrete part — no false positive
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

/** @var Box<string>&Taggable $b */
$b = new Box();

// arg type Box<string>&Taggable satisfies both parts — no error; T unbound (TIntersection arg)
$val = extract($b);
/** @mir-check $val is mixed */
echo $val;
===expect===
UnusedParam@12:18-12:29: Parameter $item is never used
