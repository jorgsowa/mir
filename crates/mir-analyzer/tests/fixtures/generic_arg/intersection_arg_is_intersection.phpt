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

// arg type Box<string>&Taggable satisfies both parts of the intersection param — no error
$val = extract($b);
===expect===
UnusedParam@12:18: Parameter $item is never used
