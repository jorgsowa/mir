===description===
subclass satisfying the concrete part via inheritance — no false positive
===file===
<?php
interface Taggable {}

class Base implements Taggable {}

/** @template T */
class Box extends Base {}

/**
 * @template T
 * @param Box<T>&Taggable $item
 * @return T
 */
function extract(mixed $item): mixed { return null; }

/** @var Box<string> $b */
$b = new Box();

// Box extends Base which implements Taggable — satisfies the concrete part
$val = extract($b);
/** @mir-check $val is string */
echo $val;
===expect===
UnusedParam@14:18-14:29: Parameter $item is never used
