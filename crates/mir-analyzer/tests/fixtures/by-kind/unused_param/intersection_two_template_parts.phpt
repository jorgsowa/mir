===description===
intersection with two template-bearing parts (Box<T>&Container<T>) — no false positive
===file===
<?php
/** @template T */
interface Container {}

/** @template T */
class Box implements Container {}

/**
 * @template T
 * @param Box<T>&Container<T> $item
 * @return T
 */
function extract(mixed $item): mixed { return null; }
//               ^^^^^^^^^^^ UnusedParam: Parameter $item is never used

/** @var Box<string> $b */
$b = new Box();

$val = extract($b);
/** @mir-check $val is string */
echo $val;
===expect===
