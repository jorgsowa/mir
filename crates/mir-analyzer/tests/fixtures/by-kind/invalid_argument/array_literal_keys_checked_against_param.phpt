===description===
An array literal's keys are checked against the parameter: an int key doesn't
fit `array<string, V>`, and a shape with a string key or with int keys out of
order isn't a `list<V>` or a list shape. A union param accepts the literal
only if one member does.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, string> $map */
function takesMap(array $map): void {}

/** @param list<string> $list */
function takesList(array $list): void {}

/** @param list{string, string} $pair */
function takesPair(array $pair): void {}

/** @param list<string>|array<string, bool|string|int> $attrs */
function takesAttrs(array $attrs): void {}

takesMap(['a' => 'x']);
takesMap([0 => 'x']);
//       ^^^^^^^^^^ InvalidArgument: Argument $map of takesMap() expects 'array<string, string>', got 'array{0: "x"}'
takesList(['a', 'b']);
takesList(['a', 'href' => 'x']);
//        ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $list of takesList() expects 'list<string>', got 'array{0: "a", 'href': "x"}'
takesPair(['x', 'y']);
takesPair([1 => 'x', 0 => 'y']);
//        ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $pair of takesPair() expects 'array{0: string, 1: string}', got 'array{1: "x", 0: "y"}'
takesAttrs(['disabled', 'contenteditable']);
takesAttrs(['disabled' => true, 'href' => 'x']);
takesAttrs(['disabled', 'href' => 'x']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $attrs of takesAttrs() expects 'list<string>|array<string, bool|string|int>', got 'array{0: "disabled", 'href': "x"}'
