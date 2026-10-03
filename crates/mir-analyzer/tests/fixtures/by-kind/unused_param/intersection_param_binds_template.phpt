===description===
intersection parameter type — InvalidArgument fires when arg violates concrete part, not when it satisfies all parts
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Box {}

interface Taggable {}

/** @template T */
class TaggedBox implements Taggable {}

/**
 * @template T
 * @param Box<T>&Taggable $item
 * @return T
 */
function extract(mixed $item): mixed { return null; }
//               ^^^^^^^^^^^ UnusedParam: Parameter $item is never used

/** @var Box<string> $b */
$b = new Box();

// Box<string> does not implement Taggable — error fires, T=string still inferred
$val = extract($b);
//             ^^ InvalidArgument: Argument $item of extract() expects 'Box<T>&Taggable', got 'Box<string>'
/** @mir-check $val is string */
echo $val;

/** @var TaggedBox<string> $tb */
$tb = new TaggedBox();

// TaggedBox<string> implements Taggable — no error; T unbound (TaggedBox != Box)
$val2 = extract($tb);
/** @mir-check $val2 is mixed */
echo $val2;
===expect===
