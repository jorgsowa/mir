===description===
intersection parameter type — T extracted from matching part, non-template part ignored
===file===
<?php
/** @template T */
class Box {}

interface Taggable {}

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
InvalidArgument@17:16: Argument $item of extract() expects 'Box<T>&Taggable', got 'Box<string>'
