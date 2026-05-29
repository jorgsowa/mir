===description===
nullable intersection param (Box<T>&Taggable)|null — no false positive when arg satisfies both parts
===file===
<?php
interface Taggable {}

/** @template T */
class Box implements Taggable {}

/**
 * @template T
 * @param (Box<T>&Taggable)|null $item
 * @return T|null
 */
function extract(mixed $item): mixed { return null; }

/** @var Box<string> $b */
$b = new Box();

// Box<string> implements Taggable and T=string — no error
$val = extract($b);
/** @mir-check $val is string|null */
echo $val;

// null matches the |null branch — no error; T unbound from null
$val2 = extract(null);
/** @mir-check $val2 is mixed */
echo $val2;
===expect===
UnusedParam@12:18-12:29: Parameter $item is never used
