===description===
intersection parameter type — InvalidArgument fires when arg violates concrete part, not when it satisfies all parts
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

// Box<string> does not implement Taggable — error should fire
$val = extract($b);

/** @template T */
class TaggedBox implements Taggable {}

/** @var TaggedBox<string> $tb */
$tb = new TaggedBox();

// TaggedBox<string> implements Taggable — no error, T binds to string
$val2 = extract($tb);
/** @mir-check $val2 is string */
===expect===
UnusedParam@12:18: Parameter $item is never used
InvalidArgument@18:16: Argument $item of extract() expects 'Box<T>&Taggable', got 'Box<string>'
